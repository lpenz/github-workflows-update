// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

type Response = serde_json::Value;

#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub client: oneshot::Sender<Result<Response>>,
}

#[derive(Debug)]
pub struct Server {
    sender: mpsc::Sender<Request>,
}

#[derive(Debug)]
pub struct Client {
    server: mpsc::Sender<Request>,
}

impl Server {
    pub fn new() -> Server {
        let (sender, mut queue): (mpsc::Sender<Request>, mpsc::Receiver<Request>) =
            mpsc::channel(32);
        tokio::spawn(async move {
            let mut cache = HashMap::<String, Response>::new();
            while let Some(req) = queue.recv().await {
                if let Some(response) = cache.get(&req.url) {
                    Server::answer(req, Ok(response.clone()));
                } else {
                    let rewsponse = reqwest::get(&req.url).await;
                    match rewsponse {
                        Ok(rewsponse) => {
                            let status = rewsponse.status();
                            if status.is_success() {
                                match rewsponse.json::<serde_json::Value>().await {
                                    Ok(json) => {
                                        cache.insert(req.url.clone(), json.clone());
                                        Server::answer(req, Ok(json))
                                    }
                                    Err(error) => Server::answer(req, Err(anyhow!(error))),
                                }
                            } else {
                                let url = req.url.clone();
                                Server::answer(
                                    req,
                                    Err(anyhow!(format!("{} while getting {}", status, url))),
                                );
                            }
                        }
                        Err(error) => Server::answer(req, Err(anyhow!(error))),
                    }
                }
            }
        });
        Server { sender }
    }

    fn answer(req: Request, response: Result<Response>) {
        req.client
            .send(response)
            .expect("error sending answer back to client");
    }

    pub fn new_client(&self) -> Client {
        Client {
            server: self.sender.clone(),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub async fn request(&self, url: String) -> Result<Response> {
        let (sender, response) = oneshot::channel();
        self.server
            .send(Request {
                url,
                client: sender,
            })
            .await?;
        response.await?
    }
}

#[tokio::test]
async fn test_basic() -> Result<()> {
    let server = Server::new();
    let client1 = server.new_client();
    let response = client1.request(String::from("file:///")).await;
    assert!(response.is_err());
    Ok(())
}
