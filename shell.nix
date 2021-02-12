let
  moz_overlay = import (builtins.fetchTarball
    "https://github.com/mozilla/nixpkgs-mozilla/archive/8c007b60731c07dd7a052cce508de3bb1ae849b4.tar.gz");

  # Nix 20.09, stable.
  pkgs = import (fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/cd63096d6d887d689543a0b97743d28995bc9bc3.tar.gz") {
      overlays = [ moz_overlay ];
    };

  rustChannel = pkgs.latest.rustChannels.stable;
in
  pkgs.mkShell {
    name = "ricochetEnv";
    buildInputs = with pkgs; [
      git
    
      # rust dependencies
      rustChannel.cargo
      rustChannel.rust

      # basic python dependencies
      python38
      python38Packages.numpy
      python38Packages.gym
      # python38Packages.pandas
      # python38Packages.scikitlearn
      # python38Packages.scipy
      # python38Packages.matplotlib

      # # a couple of deep learning libraries
      # python38Packages.tensorflow
      # python38Packages.tensorflowWithCuda # note if you get rid of WithCuda then you will not be using Cuda
      # python38Packages.Keras
      # python38Packages.pytorch # used for speedy examples
      # python38Packages.pytorchWithCuda

      # dependencies to run maturin and package ricochet_env
      maturin
      python38Packages.pip
      python38Packages.virtualenv
    ];
    NIX_ENFORCE_PURITY = 0;
    shellHook = ''
      virtualenv ricochetEnv
      source ricochetEnv/bin/activate
      unset CONDA_PREFIX # maturin wont run if conda and virtualenv are used at the same time
      maturin develop --manifest-path ricochet_environment/Cargo.toml
    '';
  }