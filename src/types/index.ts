export interface PluginInfo {
  name: string
  extensions: string[]
  capabilities: string[]
}

export interface FileInfo {
  path: string
  name: string
  size: number
  format?: string
  status: 'pending' | 'analyzing' | 'ready' | 'converting' | 'complete' | 'error'
  qualityScore?: number
  warnings?: string[]
}

export interface QualityPreview {
  score: number
  warnings: string[]
  parameters_preserved: number
  parameters_lost: number
}

export interface ConversionSettings {
  outputFormat: string
  outputPath: string
  quality: 'fast' | 'balanced' | 'maximum'
  preserveEnvelopes: boolean
  preserveFilters: boolean
  preserveLFOs: boolean
}

export interface ConversionProgress {
  currentFile: string
  filesComplete: number
  totalFiles: number
  percentage: number
  eta: number
}