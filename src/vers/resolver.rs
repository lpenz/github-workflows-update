// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{event, instrument, Level};
use versions::Version;

use crate::vers::docker;

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
        client_ch: oneshot::Sender<Result<Version>>,
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
    async fn handle_request(resource: &str, client_ch: oneshot::Sender<Result<Version>>) {
        client_ch
            .send(if let Some(url) = docker::url(resource) {
                docker::get_latest_version(&url).await
            } else {
                Err(anyhow!("could not parse resource"))
            })
            .expect("client_ch send error");
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
    pub async fn get_latest_version(&self, resource: &str) -> Result<Version> {
        let (client_ch, response) = oneshot::channel();
        self.server_ch
            .send(Message::Request {
                resource: resource.to_owned(),
                client_ch,
            })
            .await?;
        response.await?
    }
}
