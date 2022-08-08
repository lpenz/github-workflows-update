// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! A resolver [`Server`] that makes async requests and caches the
//! result, and an async [`Client`] that fills the `entity.latest`
//! field with the latest version of the upstream entity.

use anyhow;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{event, instrument, Level};

use crate::entity::Entity;
use crate::resource::Resource;
use crate::updater::updater_for;
use crate::updater::Updater;
use crate::version::Version;

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
        resource: Resource,
        client_ch: oneshot::Sender<Option<Vec<Version>>>,
    },
    Downloaded {
        resource: Resource,
        versions: Option<Vec<Version>>,
    },
}

type Cache = HashMap<Resource, Option<Vec<Version>>>;
type Pending = HashMap<Resource, Vec<oneshot::Sender<Option<Vec<Version>>>>>;

impl Server {
    #[instrument(level = "debug")]
    pub fn new() -> Server {
        let (server_ch, mut queue): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(32);
        let worker_ch = server_ch.clone();
        tokio::spawn(async move {
            event!(Level::INFO, "Server task started");
            let mut pending: Pending = Default::default();
            let mut cache: Cache = Default::default();
            while let Some(msg) = queue.recv().await {
                match msg {
                    Message::Request {
                        resource,
                        client_ch,
                    } => {
                        Server::handle_request(
                            worker_ch.clone(),
                            &cache,
                            &mut pending,
                            resource,
                            client_ch,
                        )
                        .await
                    }
                    Message::Downloaded { resource, versions } => {
                        cache.insert(resource.clone(), versions.clone());
                        if let Some(clients) = pending.remove(&resource) {
                            event!(
                                Level::INFO,
                                resource = %resource,
                                num_clients = clients.len(),
                                "retrieved, answering pending"
                            );
                            for client_ch in clients {
                                client_ch.send(versions.clone()).unwrap();
                            }
                        } else {
                            event!(
                                Level::ERROR,
                                resource = %resource,
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
        cache: &Cache,
        pending: &mut Pending,
        resource: Resource,
        client_ch: oneshot::Sender<Option<Vec<Version>>>,
    ) {
        if let Some(versions) = cache.get(&resource) {
            event!(Level::INFO, resource = %resource, "cache hit");
            Server::worker_send(worker_ch, resource, versions.clone()).await;
            return;
        }
        let updater = match updater_for(&resource) {
            Ok(updater) => updater,
            Err(e) => {
                event!(
                    Level::ERROR,
                    resource = %resource,
                    error = %e,
                    "error getting updater",
                );
                return;
            }
        };
        let url = resource.url();
        let e = pending.entry(resource.clone()).or_default();
        if e.is_empty() {
            event!(Level::INFO, resource = %resource, "downloader task started");
            tokio::spawn(async move {
                match updater.get_versions(&url).await {
                    Ok(versions) => {
                        Server::worker_send(worker_ch, resource, Some(versions)).await;
                    }
                    Err(e) => {
                        event!(
                            Level::ERROR,
                            resource = %resource,
                            updater = ?updater,
                            error = %e,
                            "error in get_version"
                        );
                        Server::worker_send(worker_ch, resource, None).await;
                    }
                };
            });
        } else {
            event!(
                Level::INFO,
                resource = %resource,
                "downloader task already present"
            );
        }
        e.push(client_ch);
    }

    #[instrument(level = "debug")]
    async fn worker_send(
        worker_ch: mpsc::Sender<Message>,
        resource: Resource,
        versions: Option<Vec<Version>>,
    ) {
        if let Err(e) = worker_ch
            .send(Message::Downloaded { resource, versions })
            .await
        {
            event!(
                Level::ERROR,
                error = %e,
                "error sending download to server task"
            );
        }
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
    pub async fn get_versions(&self, resource: &Resource) -> anyhow::Result<Option<Vec<Version>>> {
        let (client_ch, response) = oneshot::channel();
        self.server_ch
            .send(Message::Request {
                resource: resource.clone(),
                client_ch,
            })
            .await?;
        Ok(response.await?)
    }

    #[instrument(level = "debug")]
    pub async fn resolve_entity(&self, mut entity: Entity) -> Entity {
        let versions = match self.get_versions(&entity.resource).await {
            Ok(versions) => versions.unwrap_or_default(),
            Err(e) => {
                event!(
                    Level::ERROR,
                    resource = %entity.resource,
                    error = %e,
                    "error getting version",
                );
                return entity;
            }
        };
        if versions.is_empty() {
            event!(
                Level::ERROR,
                resource = %entity.resource,
                versions = ?versions,
                "no version found",
            );
            return entity;
        } else if !versions.contains(&entity.version) {
            event!(
                Level::WARN,
                resource = %entity.resource,
                current = %entity.version,
                versions = ?versions,
                "current version not present in version list",
            );
        }
        let latest = versions.iter().max().unwrap();
        event!(
            Level::INFO,
            resource = %entity.resource,
            versions = ?versions,
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
