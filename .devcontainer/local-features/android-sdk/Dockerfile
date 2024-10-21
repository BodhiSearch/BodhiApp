ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Arguments for installing dependencies where DEPENDENCIES are Tauri dependencies and EXTRA_DEPENDENCIES is empty so that users can add more without interfering with Tauri
ARG DEPENDENCIES="build-essential curl libappindicator3-dev libgtk-3-dev librsvg2-dev libssl-dev wget"
ARG EXTRA_DEPENDENCIES=""

# Argument for which image version to use
ARG IMAGE="debian"
ARG VARIANT="bullseye"

# Argument for android sdk
ARG ANDROID_SDK_TOOLS_VERSION=9477386
ARG ANDROID_PLATFORM_VERSION=34
ARG ANDROID_BUILD_TOOLS_VERSION=34.0.0
ARG NDK_VERSION=27.1.12297006

######################################
## Base image
## Installing dependencies
######################################
FROM ${IMAGE}:${VARIANT} AS base_image

# Redefine arguments
ARG DEPENDENCIES
ARG EXTRA_DEPENDENCIES

# Non-interactive frontend
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt update \
    && apt upgrade -yq \
    # Install general dependencies
    && apt install -yq --no-install-recommends sudo wget curl git xz-utils zip unzip file socat clang libssl-dev \
    pkg-config git git-lfs bash-completion llvm \
    # Install Java 17
    && apt install -yq openjdk-17-jdk \
    # Install Tauri dependencies as well as extra dependencies
    && apt install -yq ${DEPENDENCIES} ${EXTRA_DEPENDENCIES}

# Set JAVA_HOME environment variable
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH=$PATH:$JAVA_HOME/bin

# ######################################
# ## Android SDK
# ## Downloading and installing
# ######################################
# FROM base_image as android_sdk
# WORKDIR /android_sdk

# # Arguments are consumed by the first FROM they encounter, so we need to just explicitly tell Docker we need to use these again and again
# ARG ANDROID_SDK_TOOLS_VERSION
# ARG ANDROID_PLATFORM_VERSION
# ARG ANDROID_BUILD_TOOLS_VERSION
# ARG NDK_VERSION

# # Environment variables inside the android_sdk step to ensure the SDK is built properly
# ENV ANDROID_HOME="/android_sdk"
# ENV ANDROID_SDK_ROOT="$ANDROID_HOME"
# ENV NDK_HOME="${ANDROID_HOME}/ndk/${NDK_VERSION}"
# ENV PATH=${PATH}:/android_sdk/cmdline-tools/latest/bin
# ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

# # Set up the SDK
# RUN curl -C - --output android-sdk-tools.zip "https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_TOOLS_VERSION}_latest.zip" \
#     && mkdir -p /android_sdk/cmdline-tools/latest/ \
#     && unzip -q android-sdk-tools.zip -d /android_sdk/cmdline-tools/latest/ \
#     && mv /android_sdk/cmdline-tools/latest/cmdline-tools/* /android_sdk/cmdline-tools/latest \
#     && rm -r /android_sdk/cmdline-tools/latest/cmdline-tools \
#     && rm android-sdk-tools.zip \
#     && yes | sdkmanager --licenses \
#     && touch $HOME/.android/repositories.cfg \
#     && sdkmanager "cmdline-tools;latest" \
#     && sdkmanager "platform-tools" \
#     && sdkmanager "emulator" \
#     && sdkmanager "platforms;android-${ANDROID_PLATFORM_VERSION}" "build-tools;$ANDROID_BUILD_TOOLS_VERSION" \
#     && sdkmanager "ndk;${NDK_VERSION}" \
#     && sdkmanager "system-images;android-${ANDROID_PLATFORM_VERSION};google_apis;x86_64"

# # As an added bonus we set up a gradle.properties file that enhances Gradle performance
# RUN echo "org.gradle.daemon=true" >> "/gradle.properties" \
#     && echo "org.gradle.parallel=true" >> "/gradle.properties"

# ######################################
# ## Mold
# ## Speeds up compilation
# ######################################
# FROM base_image as mold
# WORKDIR /mold

# # Install dependencies
# RUN apt install -yq g++ libstdc++-10-dev zlib1g-dev cmake

# # Clone mold 1.4.2, build it then install it
# RUN git clone https://github.com/rui314/mold.git && \
#     cd mold && \
#     git checkout --quiet v1.4.2 && \
#     make -j$(nproc) CXX=clang++ && \
#     make install

# # Set up a config.toml file that makes Cargo use clang and mold
# RUN echo "[target.x86_64-unknown-linux-gnu]" >> /config.toml \
#     && echo "linker = \"clang\"" >> /config.toml \
#     && echo "rustflags = [\"-C\", \"link-arg=-fuse-ld=/usr/local/bin/mold\"]" >> /config.toml

# ######################################
# ## The finalized container
# ## Puts it all together
# ######################################
# FROM base_image

# # We require these arguments in this image so we have to fetch them anew
# ARG ANDROID_SDK_TOOLS_VERSION
# ARG ANDROID_PLATFORM_VERSION
# ARG ANDROID_BUILD_TOOLS_VERSION
# ARG NDK_VERSION

# ARG USERNAME
# ARG USER_UID
# ARG USER_GID

# # Set up the required Android environment variables
# ENV ANDROID_HOME="/home/${USERNAME}/android_sdk"
# ENV ANDROID_SDK_ROOT="$ANDROID_HOME"
# ENV NDK_HOME="${ANDROID_HOME}/ndk/${NDK_VERSION}"
# ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/emulator
# ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

# # Ensure the user is a sudo user in case the developer needs to e.g. run apt install later
# RUN echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
#     && chmod 0440 /etc/sudoers.d/$USERNAME

# # Clean up to reduce container size
# RUN apt clean \
#     && rm -rf /var/lib/apt/lists/*

# # Run the rest of the commands as the non-root user
# USER $USERNAME

# # Copy files from android_sdk
# COPY --from=android_sdk --chown=${USERNAME}:${USERNAME} /gradle.properties /home/${USERNAME}/.gradle/gradle.properties
# COPY --from=android_sdk --chown=${USERNAME}:${USERNAME} /android_sdk /home/${USERNAME}/android_sdk

# # Create an emulator
# RUN echo no | avdmanager create avd -n dev -k "system-images;android-${ANDROID_PLATFORM_VERSION};google_apis;x86_64"

# # Copy files from mold
# COPY --from=mold --chown=${USERNAME}:${USERNAME} /usr/local/bin/mold /usr/local/bin/mold
# COPY --from=mold --chown=${USERNAME}:${USERNAME} /config.toml /home/${USERNAME}/.cargo/config.toml
