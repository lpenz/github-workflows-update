// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{event, instrument, Level};
use versions::Version;

use crate::entity::Entity;
use crate::prettyvers;
use crate::updater::updater_for;
use crate::updater::Updater;

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
        client_ch: oneshot::Sender<Vec<Version>>,
    },
    Downloaded {
        resource: String,
        versions: Vec<Version>,
    },
}

type Pending = HashMap<String, Vec<oneshot::Sender<Vec<Version>>>>;

impl Server {
    #[instrument(level = "debug")]
    pub fn new() -> Server {
        let (server_ch, mut queue): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(32);
        let worker_ch = server_ch.clone();
        tokio::spawn(async move {
            event!(Level::INFO, "Server task started");
            let mut pending: Pending = Default::default();
            while let Some(msg) = queue.recv().await {
                match msg {
                    Message::Request {
                        resource,
                        client_ch,
                    } => {
                        Server::handle_request(worker_ch.clone(), &mut pending, resource, client_ch)
                            .await
                    }
                    Message::Downloaded { resource, versions } => {
                        if let Some(clients) = pending.remove(&resource) {
                            event!(
                                Level::INFO,
                                resource = resource,
                                num_clients = clients.len(),
                                "retrieved, answering pending"
                            );
                            for client_ch in clients {
                                client_ch.send(versions.clone()).unwrap();
                            }
                        } else {
                            event!(
                                Level::ERROR,
                                resource = resource,
                                "no pending request found"
                            );
                        }
                    }
                }
            }
        });
        Server { server_ch }
    }

    #[instrument(level = "debug")]
    async fn handle_request(
        worker_ch: mpsc::Sender<Message>,
        pending: &mut Pending,
        resource: String,
        client_ch: oneshot::Sender<Vec<Version>>,
    ) {
        let updater = match updater_for(&resource) {
            Ok(updater) => updater,
            Err(e) => {
                event!(
                    Level::ERROR,
                    resource = resource,
                    error = %e,
                    "error getting updater",
                );
                return;
            }
        };
        let url = match updater.url(&resource) {
            Some(url) => url,
            None => {
                event!(
                    Level::ERROR,
                    resource = resource,
                    updater = ?updater,
                    "updater could not parse url",
                );
                return;
            }
        };
        let e = pending.entry(resource.clone()).or_default();
        if e.is_empty() {
            event!(Level::INFO, resource = resource, "downloader task started");
            tokio::spawn(async move {
                match updater.get_versions(&url).await {
                    Ok(versions) => {
                        worker_ch
                            .send(Message::Downloaded { resource, versions })
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        event!(
                            Level::ERROR,
                            url = url,
                            updater = ?updater,
                            error = %e,
                            "error in get_version"
                        );
                    }
                };
            });
        } else {
            event!(
                Level::INFO,
                resource = resource,
                "downloader task already present"
            );
        }
        e.push(client_ch);
    }

    #[instrument(level = "debug")]
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
    #[instrument(level = "debug")]
    pub async fn get_versions(&self, resource: &str) -> Result<Vec<Version>> {
        let (client_ch, response) = oneshot::channel();
        self.server_ch
            .send(Message::Request {
                resource: resource.to_owned(),
                client_ch,
            })
            .await?;
        Ok(response.await?)
    }

    #[instrument(level = "debug")]
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
                versions = ?prettyvers::Versions::new(&versions),
                "no version found",
            );
            return entity;
        } else if !versions.contains(&entity.version) {
            event!(
                Level::WARN,
                resource = entity.resource,
                current = %entity.version,
                versions = ?prettyvers::Versions::new(&versions),
                "current version not present in version list",
            );
        }
        let latest = versions.iter().max().unwrap();
        event!(
            Level::INFO,
            resource = entity.resource,
            versions = ?prettyvers::Versions::new(&versions),
            latest = %latest,
            "got versions",
        );
        entity.latest = Some(latest.clone());
        if let Ok(updater) = updater_for(&entity.resource) {
            entity.updated_line = updater.updated_line(&entity);
        }
        entity
    }
}
