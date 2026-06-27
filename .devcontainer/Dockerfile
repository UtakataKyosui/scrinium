FROM mcr.microsoft.com/devcontainers/base:ubuntu

ARG USERNAME=utakata
ARG USER_UID=1100
ARG USER_GID=${USER_UID}

RUN groupadd --gid ${USER_GID} ${USERNAME} \
     && useradd -m -s /bin/bash --uid ${USER_UID} --gid ${USER_GID} ${USERNAME} \
     && apt-get update \
     && apt-get install -y sudo curl xz-utils ca-certificates build-essential \
     && echo "${USERNAME} ALL=(root) NOPASSWD:ALL" >/etc/sudoers.d/${USERNAME} \
     && chmod 0440 /etc/sudoers.d/${USERNAME}

USER ${USERNAME}

RUN curl -L https://releases.nixos.org/nix/nix-2.30.1/install | sh -s -- --no-daemon

ENV PATH="/home/${USERNAME}/.nix-profile/bin:${PATH}"

# Configure binary caches BEFORE installing any packages so all downloads benefit from them
RUN mkdir -p ~/.config/nix \
     && printf '%s\n' \
          'experimental-features = nix-command flakes' \
          'substituters = https://cache.nixos.org https://nix-community.cachix.org' \
          'trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCUSBD4= rust-overlay.cachix.org-1:fnlXmtnCVHkn64x3JuWCRKRiKXgdPNcLgD7ICxZG8jE=' \
          'max-jobs = auto' \
          'cores = 0' \
          'http-connections = 128' \
          'keep-derivations = true' \
          'keep-outputs = true' \
          > ~/.config/nix/nix.conf

# Install direnv + nix-direnv globally so devshell activations are cached
RUN nix profile install nixpkgs#direnv nixpkgs#nix-direnv

RUN echo 'eval "$(direnv hook bash)"' >> /home/${USERNAME}/.bashrc \
     && mkdir -p /home/${USERNAME}/.config/direnv \
     && echo 'source $HOME/.nix-profile/share/nix-direnv/direnvrc' \
          > /home/${USERNAME}/.config/direnv/direnvrc

# Pre-populate the Nix store at image build time so container startup is instant
COPY --chown=${USERNAME}:${USERNAME} flake.nix flake.lock /tmp/devshell-cache/
RUN cd /tmp/devshell-cache \
     && nix develop --no-write-lock-file --command true \
     && rm -rf /tmp/devshell-cache
