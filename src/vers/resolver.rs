// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{event, instrument, Level};
use versions::Version;

use crate::entity::Entity;
use crate::vers::docker;
use crate::vers::Versions;

#[derive(Debug)]
pub struct Server {
    server_ch: mpsc::Sender<Message>,
}

#[derive(Debug)]
pub struct Client {
    server_ch: mpsc::Sender<Message>,
}

#[derive(Debug)]
pub enum Message {
    Request {
        resource: String,
        client_ch: oneshot::Sender<Result<Vec<Version>>>,
    },
}

impl Server {
    #[instrument]
    pub fn new() -> Server {
        let (server_ch, mut queue): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(32);
        tokio::spawn(async move {
            event!(Level::INFO, "Server task started");
            while let Some(msg) = queue.recv().await {
                match msg {
                    Message::Request {
                        resource,
                        client_ch,
                    } => Server::handle_request(&resource, client_ch).await,
                }
            }
        });
        Server { server_ch }
    }

    #[instrument]
    async fn handle_request(resource: &str, client_ch: oneshot::Sender<Result<Vec<Version>>>) {
        if let Some(url) = docker::url(resource) {
            client_ch.send(docker::get_versions(&url).await).unwrap();
        }
    }

    #[instrument]
    pub fn new_client(&self) -> Client {
        Client {
            server_ch: self.server_ch.clone(),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    #[instrument]
    pub async fn get_versions(&self, resource: &str) -> Result<Vec<Version>> {
        let (client_ch, response) = oneshot::channel();
        self.server_ch
            .send(Message::Request {
                resource: resource.to_owned(),
                client_ch,
            })
            .await?;
        response.await?
    }

    #[instrument]
    pub async fn resolve_entity(&self, mut entity: Entity) -> Entity {
        let versions = match self.get_versions(&entity.resource).await {
            Ok(versions) => versions,
            Err(e) => {
                event!(
                    Level::ERROR,
                    resource = entity.resource,
                    error = %e,
                    "error getting version",
                );
                return entity;
            }
        };
        if versions.is_empty() {
            event!(
                Level::ERROR,
                resource = entity.resource,
                versions = ?Versions::new(&versions),
                "no version found",
            );
            return entity;
        } else if !versions.contains(&entity.version) {
            event!(
                Level::WARN,
                resource = entity.resource,
                current = %entity.version,
                versions = ?Versions::new(&versions),
                "current version not present in version list",
            );
        }
        let latest = versions.iter().max().unwrap();
        event!(
            Level::INFO,
            resource = entity.resource,
            versions = ?Versions::new(&versions),
            latest = %latest,
            "got versions",
        );
        entity.latest = Some(latest.clone());
        entity
    }
}
