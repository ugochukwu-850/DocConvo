
use anyhow::Context;
use pdf_extract::extract_text;
use rig::{
    completion::Prompt, embeddings::EmbeddingsBuilder,
    vector_store::VectorStoreIndex,
};
use std::{
    fs,
    path::{Path, PathBuf}
};
use tauri::State;

use crate::AppState;
use walkdir::WalkDir;
use rig::providers::gemini::completion::GEMINI_1_5_FLASH;


pub fn load_folder_contents<P: AsRef<Path>>(dir_path: P) -> anyhow::Result<Vec<String>> {
    let mut contents = Vec::new();

    for entry in WalkDir::new(dir_path) {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("txt") | Some("srt") | Some("rs") => {
                    let text = fs::read_to_string(path)
                        .with_context(|| format!("Failed to read text file: {:?}", path))?;
                    contents.push(text);
                }
                Some("pdf") => {
                    let pdf_text = extract_text(path)
                        .with_context(|| format!("Failed to extract text from PDF: {:?}", path))?;
                    println!("PDF TEXT \n {:?}", pdf_text);

                    contents.push(pdf_text);
                }
                _ => {
                    // Unsupported file types are ignored
                }
            }
        }
    }

    Ok(contents)
}


#[tauri::command]
pub async fn prompt(prompt: String, state: State<'_, AppState>) -> Result<String, ()> {
    let vector_store_guard = state.vector_store.lock().await;
    let client = &state.client;
    let model = client.embedding_model("embedding-001");
    let ind = vector_store_guard
        .clone()
        .index(model.clone())
        .top_n::<String>(&prompt, 5)
        .await
        .unwrap()
        .iter()
        .map(|f| f.2.clone())
        .collect::<Vec<_>>();

    println!("{:?}", ind);

    let rag_response = client.agent(GEMINI_1_5_FLASH).preamble("You are a helpful assistant that answers questions based on the given context from documents.").context(&ind.join(",")).build().prompt(&prompt).await.unwrap();

    Ok(rag_response)
}

#[tauri::command]
pub async fn index_folders(folders: Vec<PathBuf>, state: State<'_, AppState>) -> Result<(), ()> {
    let documents = folders
        .into_iter()
        .filter_map(|f| load_folder_contents(f).ok())
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(",");

    println!("{:?}", documents);

    let mut vector_store_guard = state.vector_store.lock().await;
    let client = &state.client;

    let model = client.embedding_model("embedding-001");
    let embeddings = EmbeddingsBuilder::new(model.clone())
        .document(documents)
        .unwrap()
        .build()
        .await
        .unwrap();
    vector_store_guard.add_documents(embeddings);
    Ok(())
}
