# Setup Wizard: LLM Engine Selection

## User Story

As a Bodhi App user,
I want to select and download the optimal LLM inference engine for my hardware,
So that I can get the best performance for my system.

## Background

- Different inference engines optimize for different hardware
- Hardware analysis determines best engine recommendation
- Fallback CPU option always available
- Step can be skipped and completed later
- Downloads are typically under 100MB

## Acceptance Criteria

### Hardware Analysis Display

- [x] System capability summary card
- [x] Expandable technical details
- [x] Detection of:
  - Operating System
  - CPU capabilities (AVX, etc.)
  - GPU manufacturer and memory
  - Available RAM
  - GPU drivers (CUDA, Vulkan)
- [x] User-friendly status indicators
- [x] Fallback if detection fails

### Engine Recommendations

- [x] Primary recommended engine
- [x] Alternative options list
- [x] Hardware compatibility indicators
- [x] Download size information
- [x] CPU fallback option
- [x] Skip option availability

### Download Process

- [x] Progress indication
- [x] 4 retry attempts on failure
- [x] Error messages
- [x] Space requirement check
- [x] Compatibility verification
- [x] Skip/Retry options

## Content Structure

### Layout

```
Desktop Layout (>768px):
┌─────────────────────────────────┐
│      Setup Progress (4/5)       │
├─────────────────────────────────┤
│    Hardware Analysis Card       │
│ [Click to Show Technical Info]  │
├─────────────────────────────────┤
│    Recommended Engine Card      │
├─────────────────────────────────┤
│    Alternative Options          │
├─────────────────────────────────┤
│    Download Progress/Actions    │
└─────────────────────────────────┘

Mobile Layout (<768px):
┌────────────────────┐
│  Progress (4/5)    │
├────────────────────┤
│ Hardware Analysis  │
│ [Expand Details]   │
├────────────────────┤
│  Recommendation    │
├────────────────────┤
│ Other Options      │
├────────────────────┤
│ Download/Actions   │
└────────────────────┘
```

### Content Sections

#### Hardware Analysis Card

```
System Capabilities Summary
-------------------------
OS: Windows 11 Pro
GPU: NVIDIA RTX 4080 (12GB)
CPU: Intel i7 (AVX2 Support)
RAM: 32GB Available

[Show Technical Details ▼]

Detailed Analysis (Expanded View)
-------------------------------
GPU Driver: CUDA 11.8 installed
CPU Extensions: AVX, AVX2, SSE4.1, SSE4.2
GPU Compute: Supported
System Architecture: x86_64
... (additional technical details)
```

#### Engine Recommendations

```
Recommended Engine
----------------
CUDA-Optimized Engine
Optimal for your NVIDIA GPU
Download Size: 85MB

Alternative Options
----------------
1. CPU+GPU Hybrid (75MB)
2. CPU-Optimized (60MB)
3. Generic CPU (45MB)

Skip Option
----------
Skip for now (can be downloaded later from settings)
```

#### Download Progress

```
Downloading: CUDA-Optimized Engine
Progress: [==========] 85%
Speed: 2.5MB/s
Estimated time: 30s
```

#### Error Messages

```
Download Failed:
"Connection interrupted. Retry attempt 1/4..."

Space Warning:
"Insufficient disk space. 100MB required."

Compatibility Error:
"Selected engine not compatible. 
 Switching to generic CPU version."
```

## Technical Details

### Component Structure

```typescript
interface HardwareAnalysis {
  os: SystemInfo;
  cpu: CPUInfo;
  gpu: GPUInfo;
  ram: MemoryInfo;
  drivers: DriverInfo;
}

interface EngineOption {
  id: string;
  name: string;
  size: number;
  compatibility: string[];
  isRecommended: boolean;
}
```

### State Management

```typescript
interface EngineSetupState {
  analysisComplete: boolean;
  selectedEngine: string;
  downloadProgress: number;
  retryCount: number;
  error?: string;
}
```

## Testing Criteria

### Hardware Detection

- OS detection accuracy
- GPU detection reliability
- CPU capability detection
- Driver detection
- Fallback handling

### Download Process

- Progress tracking
- Retry mechanism
- Error handling
- Space verification
- Compatibility check

### UI/UX

- Responsive layouts
- Expandable details
- Progress indicators
- Error states
- Loading states

### Accessibility

- Screen reader support
- Keyboard navigation
- Focus management
- Status announcements

## Out of Scope

- Engine performance metrics
- Detailed compatibility tests
- Advanced hardware analysis
- Custom engine configurations
- Performance comparisons

## Dependencies

- Hardware analysis system
- Download manager
- Storage manager
- Engine compatibility checker
