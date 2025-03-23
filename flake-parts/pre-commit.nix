{
  perSystem = {self', ...}: {
    pre-commit = {
      check.enable = true;

      settings = {
        src = ../.;
        hooks = {
          treefmt = {
            enable = true;
            package = self'.packages.treefmt;
          };
        };
      };
    };
  };
}
