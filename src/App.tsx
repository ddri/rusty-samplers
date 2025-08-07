import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import FilePanel from './components/FilePanel'
import ControlPanel from './components/ControlPanel'
import QualityPanel from './components/QualityPanel'
import { PluginInfo, FileInfo, QualityPreview } from './types'
import './App.css'

function App() {
  const [plugins, setPlugins] = useState<PluginInfo[]>([])
  const [files, setFiles] = useState<FileInfo[]>([])
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null)
  const [qualityPreview, setQualityPreview] = useState<QualityPreview | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  // Load available plugins on startup
  useEffect(() => {
    const loadPlugins = async () => {
      try {
        const availablePlugins: PluginInfo[] = await invoke('get_available_plugins')
        setPlugins(availablePlugins)
      } catch (error) {
        console.error('Failed to load plugins:', error)
      }
    }
    
    loadPlugins()
  }, [])

  // Analyze file quality when file is selected
  useEffect(() => {
    if (selectedFile) {
      const analyzeFile = async () => {
        setIsLoading(true)
        try {
          const quality: QualityPreview = await invoke('analyze_file_quality', { 
            filePath: selectedFile.path 
          })
          setQualityPreview(quality)
        } catch (error) {
          console.error('Failed to analyze file quality:', error)
        } finally {
          setIsLoading(false)
        }
      }
      
      analyzeFile()
    }
  }, [selectedFile])

  const handleFilesAdded = (newFiles: FileInfo[]) => {
    setFiles(prev => [...prev, ...newFiles])
  }

  const handleFileSelected = (file: FileInfo) => {
    setSelectedFile(file)
  }

  const handleConversion = async (outputFormat: string) => {
    if (!selectedFile) return
    
    setIsLoading(true)
    try {
      const result: string = await invoke('start_conversion', {
        inputPath: selectedFile.path,
        outputPath: selectedFile.path.replace(/\.[^/.]+$/, `.${outputFormat.toLowerCase()}`),
        format: outputFormat
      })
      console.log('Conversion result:', result)
    } catch (error) {
      console.error('Conversion failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Rusty Samplers</h1>
        <div className="header-info">
          <span className="version">v0.9.0</span>
          <span className="plugins-count">{plugins.length} plugins loaded</span>
        </div>
      </header>

      <main className="app-main">
        <div className="panels-container">
          {/* File Management Panel */}
          <div className="panel file-panel">
            <FilePanel 
              files={files}
              selectedFile={selectedFile}
              onFilesAdded={handleFilesAdded}
              onFileSelected={handleFileSelected}
            />
          </div>

          {/* Control Panel */}
          <div className="panel control-panel">
            <ControlPanel
              plugins={plugins}
              selectedFile={selectedFile}
              isLoading={isLoading}
              onConversion={handleConversion}
            />
          </div>

          {/* Quality Panel */}
          <div className="panel quality-panel">
            <QualityPanel
              selectedFile={selectedFile}
              qualityPreview={qualityPreview}
              isLoading={isLoading}
            />
          </div>
        </div>
      </main>

      <footer className="app-footer">
        <span>Professional audio format conversion platform</span>
        <div className="footer-links">
          <a href="#" onClick={() => console.log('Help clicked')}>Help</a>
          <a href="#" onClick={() => console.log('About clicked')}>About</a>
        </div>
      </footer>
    </div>
  )
}

export default App