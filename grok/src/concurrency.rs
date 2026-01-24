use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct ActorSystem {
    actors: HashMap<String, mpsc::Sender<Message>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Data(String),
    // Add more message types
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }

    pub async fn spawn_actor<F>(&mut self, name: String, handler: F)
    where
        F: Fn(Message) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(32);
        self.actors.insert(name.clone(), tx);

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                handler(msg);
            }
        });
    }

    pub async fn send(&self, actor: &str, msg: Message) -> Result<(), String> {
        if let Some(tx) = self.actors.get(actor) {
            tx.send(msg).await.map_err(|_| "Send failed".to_string())
        } else {
            Err("Actor not found".to_string())
        }
    }
}
