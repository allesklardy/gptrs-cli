use chatgpt::prelude::*;
use chatgpt::types::CompletionResponse;
use dotenv::dotenv;
use termimad::crossterm::style::Color::*;
use text_io::read;
use tokio::sync::*;

#[tokio::main]
async fn main() {
    let (input_tx, input_rx) = broadcast::channel::<String>(10);
    let (output_tx, output_rx) = broadcast::channel::<String>(10);

    spawn_chatgpt(input_rx, output_tx).await;
    spawn_output(output_rx).await;
    loop {
        let prompt: String = read!("{}\n");
        input_tx.send(prompt).unwrap();
    }
}

async fn spawn_output(mut output_rx: broadcast::Receiver<String>) {
    tokio::spawn(async move {
        let mut skin = termimad::MadSkin::default();
        skin.bold.set_fg(Yellow);
        for i in 0..8 {
            skin.headers[i].set_fg(Red);
        }

        loop {
            let response: String = output_rx.recv().await.unwrap();
            skin.print_text(&response);
        }
    });
}

async fn spawn_chatgpt(
    mut input_rx: broadcast::Receiver<String>,
    output_tx: broadcast::Sender<String>,
) {
    tokio::spawn(async move {
        dotenv().ok();
        let key = std::env::var("KEY").expect("KEY not set");

        let client = ChatGPT::new_with_config(
            key,
            ModelConfigurationBuilder::default()
                .temperature(1.0)
                .engine(ChatGPTEngine::Gpt35Turbo)
                .build()
                .unwrap(),
        )
        .unwrap();
        let mut conversation: Conversation = client.new_conversation();

        loop {
            let prompt: String = input_rx.recv().await.unwrap();

            let response: CompletionResponse = conversation.send_message(prompt).await.unwrap();
            output_tx.send(response.message().content.clone()).unwrap();
        }
    });
}
