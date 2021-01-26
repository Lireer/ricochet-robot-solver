let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    name = "ricochetEnv";
    buildInputs = with pkgs; [
    # basic python dependencies
      python38
      python38Packages.numpy
      python38Packages.gym
    #   python38Packages.pandas
    #   python38Packages.scikitlearn
    #   python38Packages.scipy
    #   python38Packages.matplotlib

      maturin

    # # a couple of deep learning libraries
    # #  python38Packages.tensorflow
    #   python38Packages.tensorflowWithCuda # note if you get rid of WithCuda then you will not be using Cuda
    #   python38Packages.Keras
    # #  python38Packages.pytorch # used for speedy examples
    #   python38Packages.pytorchWithCuda

    # virtualenv dependencies
      python38Packages.pip
      python38Packages.virtualenv
    ];
    shellHook = ''
      virtualenv ricochetEnv
      source ricochetEnv/bin/activate
      # pip install -r requirements.txt 
    '';
  }