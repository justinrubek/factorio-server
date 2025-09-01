{
  perSystem = {inputs', ...}: {
    packages = {
      inherit (inputs'.vintagestory.packages) vintagestory;
    };
  };
}
