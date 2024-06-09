/*
// TODO: replace test
#[rstest]
#[tokio::test]
async fn test_interactive_chat_template_not_found() -> anyhow::Result<()> {
  let alias = Alias::testalias();
  let mut mock = MockAppService::default();
  mock
    .expect_find_local_file()
    .with(
      eq(alias.repo.clone()),
      eq(alias.filename.clone()),
      eq(alias.snapshot.clone()),
    )
    .return_once(|_, _, _| Ok(Some(LocalModelFile::testalias())));
  let llama3 = Repo::try_new("meta-llama/Meta-Llama-3-8B-Instruct".to_string())?;
  mock
    .expect_find_local_file()
    .with(eq(llama3.clone()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
    .return_once(|_, _, _| Ok(None));
  mock
    .expect_model_file_path()
    .with(eq(llama3), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
    .return_once(|_, _, _| {
      PathBuf::from(
        "/tmp/huggingface/hub/models--meta-llama-repo/snapshots/xyz/tokenizer_config.json",
      )
    });
  let result = Interactive::new(alias).execute(&mock).await;
  assert!(result.is_err());
  assert_eq!(
    r#"model files for model alias 'testalias:instruct' not found in huggingface cache directory. Check if file in the expected filepath exists.
filepath: /tmp/huggingface/hub/models--meta-llama-repo/snapshots/xyz/tokenizer_config.json
"#,
    result.unwrap_err().to_string()
  );
  Ok(())
}

  #[rstest]
  #[tokio::test]
  async fn test_interactive_chat_with_llama3(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let AppServiceTuple(_temp_bodhi, _temp_hf, _bodhi_home, _hf_cache, service) = app_service_stub;
    let handle = tokio::spawn(async move {
      let alias = Alias::tinyllama();
      Interactive::new(alias).execute(&service).await
    });
    todo!()
  }

*/
