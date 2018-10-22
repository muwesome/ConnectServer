pub trait RpcServiceConfig: Send + Sync + 'static {
  fn host(&self) -> &str;

  fn port(&self) -> u16;
}
