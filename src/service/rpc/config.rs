#[cfg(feature = "build-binary")]
use structopt::StructOpt;

#[cfg_attr(feature = "build-binary", derive(StructOpt))]
pub struct RpcServiceConfig {
  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "rpc-host",
      help = "Bind to this RPC domain",
      default_value = "0.0.0.0"
    )
  )]
  pub rpc_host: String,

  #[cfg_attr(
    feature = "build-binary",
    structopt(
      long = "rpc-port",
      help = "Bind to this RPC listener port",
      default_value = "0"
    )
  )]
  pub rpc_port: u16,
}
