use anyhow::Result;
use rquickjs::{AsyncContext, AsyncRuntime, Function, Value};
use std::sync::mpsc;
use tokio::sync::oneshot;

type Request = (String, oneshot::Sender<Result<String>>);

pub struct JsWorker {
    sender: mpsc::Sender<Request>,
}

impl JsWorker {
    pub async fn new(source: String) -> Result<Self> {
        let (sender, receiver) = mpsc::channel::<Request>();

        std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(value) => value,
                Err(_) => return,
            };

            runtime.block_on(async move {
                let quick_runtime = match AsyncRuntime::new() {
                    Ok(value) => value,
                    Err(_) => return,
                };

                let context = match AsyncContext::full(&quick_runtime).await {
                    Ok(value) => value,
                    Err(_) => return,
                };

                if context.with(|ctx| ctx.eval::<(), _>(source)).await.is_err() {
                    return;
                }

                while let Ok((request, response)) = receiver.recv() {
                    let result: Result<String, rquickjs::Error> = context
                        .with(|ctx| {
                            let handler: Function = ctx.globals().get("handler")?;
                            let value: Value = handler.call((request,))?;

                            match value.as_string() {
                                Some(value) => Ok(value.to_string()?),
                                None => Ok(String::new()),
                            }
                        })
                        .await;

                    let _ = response.send(result.map_err(anyhow::Error::from));
                }
            });
        });

        Ok(Self { sender })
    }

    pub async fn execute(&self, request: String) -> Result<String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender.send((request, response_tx))?;

        response_rx.await?
    }
}
