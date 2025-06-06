# Download Llama Server Variants

## Overview
Enhance the Bodhi App release, by downloading pre-built binaries from GitHub releases as part of packaging, allowing to package GPU specific optimized llama-server builds
We also want to enhance the overall way in which these llama-server binaries are handled in the app. Now we will have 2 types of llama-server binaries -
- resource binaries - comes packaged with the app
- BODHI_HOME binaries - downloaded by the user or the app after the app is installed
this allows us the limit the size of release package, as well as give flexibility to allow user to select and download variant most optimized for his machine.

## Background
Currently, the llama server variants are handled through BUILD_VARIANTS and are bundled resources. 
We need to support downloading pre-built binaries and organize variants based on their location (resource bundle vs BODHI_HOME).

## Requirements

### Download Pre-built Binaries
- When CI_RELEASE is enabled, fetch llama server binaries from GitHub releases
- Check latest release of BodhiApp/llama.cpp repository
- Download artifacts using pattern: llama-server--{TARGET}--{VARIANT}.zip
- Extract downloaded zip to temp folder
- Copy llama-server binary to crates/llama_server_proc/bin/{TARGET}/{VARIANT}/llama-server{EXTENSION}

### Variant Management
- Separate variants into two categories:
  - Resource variants: Bundled with the application
  - BODHI_HOME variants: User-downloaded variants
- Update settings service to:
  - List resource variants with path RESOURCE/bin/{TARGET}/{variant}
  - List BODHI_HOME variants with path BODHI_HOME/bin/{TARGET}/{variant}
- Remove direct usage of llama_server_proc::BUILD_VARIANTS

## Tasks
1. Update build.rs:
   - [ ] Add GitHub release API integration
   - [ ] Implement artifact download logic
   - [ ] Add zip extraction functionality
   - [ ] Implement binary copy to target location
   - [ ] Add CI_RELEASE conditional logic

2. Update Variant Management:
   - [ ] Create new enum/struct for variant types
   - [ ] Implement resource variant discovery
   - [ ] Implement BODHI_HOME variant discovery
   - [ ] Update settings service to handle both variant types

3. Testing:
   - [ ] Add tests for running resource variant
   - [ ] Add tests for running BODHI_HOME variant
   - [ ] Add tests for listing resource variant
   - [ ] Add tests for listing BODHI_HOME variant
   - [ ] Add tests for combining the listing and allowing user to select

4. Documentation:
   - [ ] Document new variant management system
   - [ ] Update build documentation
   - [ ] Add examples for custom variant installation

## Questions/Clarifications Needed

1. Purpose Questions:
   - What is the primary goal of downloading pre-built binaries instead of building from source?
   = for development, the default binary is built as part of the llama_server_proc compilation process. So having multiple environments is not possible at the same time.
     for e.g. having CUDA and Vulkan etc. are not present at the same time. So having the pre-built standalone binaries from a separate build process are copied.
   - How does this improve the user experience?
   = decreases the size of build. also allows flexibility to download variants specific for their architecture later.
   - What are the security implications of downloading pre-built binaries?
   = as the binaries are built by our own Github workflows, they are secure. But user can modify and put a binary later, that will be their choice.

2. Technical Questions:
   - What is the fallback mechanism if GitHub releases are unavailable?
   = None
   - Should we implement version checking for downloaded variants?
   = Later
   - How should we handle binary signature verification?
   = Later
   - What is the cleanup strategy for temporary download files?
   = This runs on CI, so the whole instance is recycled after the process is over
   - How do we handle download failures during CI?
   = Fail the build

3. User Experience Questions:
   - How should we communicate to users which variants are bundled vs downloaded?
   = Later
   - Should we provide a UI for managing downloaded variants?
   = Later
   - What happens if a user tries to use a variant that isn't downloaded?
   = We get a executable not found exception

4. Implementation Questions:
   - Should we implement download progress indicators?
   = This happens on CI. So no need for indicators. We are not having the user downloads feature now, we will implement it when we do that.
   - How do we handle platform-specific binary extensions?
   = So far for windows we have the extension `.exe`, in build.rs we have mechanism to handle this.
   - What is the strategy for handling binary updates?
   = Later
   - Should we implement parallel downloads for multiple variants?
   = Lets keep it simple and download one at a time

## Success Criteria
- CI can successfully download and package llama server binaries
- Settings correctly display both resource and BODHI_HOME variants
- Users can seamlessly use both bundled and downloaded variants
- Build process handles all supported platforms and variants correctly 
