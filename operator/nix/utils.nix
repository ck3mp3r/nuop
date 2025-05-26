let
  systemMap = sys: let
    parts = builtins.match "([a-z0-9_]+)-([a-z]+)" sys;
  in {
    arch = builtins.elemAt parts 0;
    platform = builtins.elemAt parts 1;
  };

  getTarget = system:
    {
      "aarch64-darwin" = "aarch64-apple-darwin";
      "aarch64-linux" = "aarch64-unknown-linux-musl";
      "x86_64-linux" = "x86_64-unknown-linux-musl";
    }
    .${system};
in {
  inherit systemMap getTarget;
}
