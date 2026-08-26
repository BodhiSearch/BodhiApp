export interface DockerVariantMetadata {
  name: string;
  displayName: string;
  description: string;
  icon: 'cpu' | 'gpu-nvidia' | 'gpu-amd' | 'gpu-generic';
  gpuVendor?: 'NVIDIA' | 'AMD' | 'Intel' | 'Cross-vendor' | 'Huawei Ascend' | 'Moore Threads';
  recommended?: boolean;
  color: string; // Tailwind color class (without prefix)
}

export const VARIANT_METADATA: Record<string, DockerVariantMetadata> = {
  cpu: {
    name: 'cpu',
    displayName: 'CPU',
    description: 'Multi-platform CPU variant (AMD64 + ARM64)',
    icon: 'cpu',
    color: 'blue',
  },
  cuda: {
    name: 'cuda',
    displayName: 'CUDA',
    description: 'NVIDIA GPU acceleration (8-12x faster)',
    icon: 'gpu-nvidia',
    gpuVendor: 'NVIDIA',
    color: 'green',
  },
  rocm: {
    name: 'rocm',
    displayName: 'ROCm',
    description: 'AMD GPU acceleration',
    icon: 'gpu-amd',
    gpuVendor: 'AMD',
    color: 'red',
  },
  vulkan: {
    name: 'vulkan',
    displayName: 'Vulkan',
    description: 'Cross-vendor GPU acceleration',
    icon: 'gpu-generic',
    gpuVendor: 'Cross-vendor',
    color: 'purple',
  },
  intel: {
    name: 'intel',
    displayName: 'Intel',
    description: 'Intel GPU acceleration (SYCL/OneAPI)',
    icon: 'gpu-generic',
    gpuVendor: 'Intel',
    color: 'indigo',
  },
  cann: {
    name: 'cann',
    displayName: 'CANN',
    description: 'Huawei Ascend NPU acceleration (AMD64 + ARM64)',
    icon: 'gpu-generic',
    gpuVendor: 'Huawei Ascend',
    color: 'orange',
  },
  musa: {
    name: 'musa',
    displayName: 'MUSA',
    description: 'Moore Threads GPU acceleration',
    icon: 'gpu-generic',
    gpuVendor: 'Moore Threads',
    color: 'teal',
  },
};

export function getVariantMetadata(variantKey: string): DockerVariantMetadata {
  return (
    VARIANT_METADATA[variantKey] || {
      name: variantKey,
      displayName: variantKey.toUpperCase(),
      description: `${variantKey} variant`,
      icon: 'cpu',
      color: 'gray',
    }
  );
}
