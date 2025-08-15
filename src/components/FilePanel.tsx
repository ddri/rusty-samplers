import React, { useCallback } from 'react'
import { Upload, FileText, AlertTriangle, CheckCircle, Clock } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { FileInfo } from '../types'

interface FilePanelProps {
  files: FileInfo[]
  selectedFile: FileInfo | null
  onFilesAdded: (files: FileInfo[]) => void
  onFileSelected: (file: FileInfo) => void
}

const FilePanel: React.FC<FilePanelProps> = ({
  files,
  selectedFile,
  onFilesAdded,
  onFileSelected
}) => {
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    
    // Get file paths from the dropped files
    const droppedFiles = Array.from(e.dataTransfer.files)
    const fileInfos: FileInfo[] = []
    
    for (const file of droppedFiles) {
      // In Tauri, we need to get the actual file path
      // For now, create FileInfo with available data
      const fileInfo: FileInfo = {
        path: (file as any).path || file.name, // file.path in Tauri context
        name: file.name,
        size: file.size,
        format: file.name.split('.').pop()?.toUpperCase(),
        status: 'pending'
      }
      fileInfos.push(fileInfo)
    }
    
    onFilesAdded(fileInfos)
  }, [onFilesAdded])

  const handleBrowseFiles = useCallback(async () => {
    try {
      const selectedPaths = await open({
        multiple: true,
        filters: [{
          name: 'Akai Program Files',
          extensions: ['akp']
        }]
      })
      
      if (selectedPaths && Array.isArray(selectedPaths)) {
        const fileInfos: FileInfo[] = selectedPaths.map(path => ({
          path: path,
          name: path.split(/[/\\]/).pop() || path,
          size: 0, // We'll need to get this from the backend if needed
          format: 'AKP',
          status: 'pending'
        }))
        
        onFilesAdded(fileInfos)
      }
    } catch (error) {
      console.error('Failed to open file dialog:', error)
    }
  }, [onFilesAdded])

  const getStatusIcon = (status: FileInfo['status']) => {
    switch (status) {
      case 'ready':
        return <CheckCircle className="status-icon success" size={16} />
      case 'analyzing':
        return <Clock className="status-icon info" size={16} />
      case 'error':
        return <AlertTriangle className="status-icon error" size={16} />
      default:
        return <FileText className="status-icon" size={16} />
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  return (
    <div className="file-panel-container">
      <div className="panel-header">
        <h2>Files</h2>
        <span className="file-count">{files.length} files</span>
      </div>

      {/* Drag & Drop Zone */}
      <div 
        className="drop-zone"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={handleBrowseFiles}
      >
        <Upload size={32} className="upload-icon" />
        <p className="drop-text">
          Drop AKP files here
        </p>
        <p className="drop-subtext">
          or click to browse
        </p>
      </div>

      {/* File List */}
      <div className="file-list">
        {files.length === 0 ? (
          <div className="empty-state">
            <FileText size={48} className="empty-icon" />
            <p>No files added yet</p>
            <p className="empty-subtext">
              Drag and drop AKP files to get started
            </p>
          </div>
        ) : (
          files.map((file, index) => (
            <div 
              key={index}
              className={`file-item ${selectedFile === file ? 'selected' : ''}`}
              onClick={() => onFileSelected(file)}
            >
              <div className="file-info">
                <div className="file-header">
                  {getStatusIcon(file.status)}
                  <span className="file-name">{file.name}</span>
                  {file.format && (
                    <span className="format-badge">{file.format}</span>
                  )}
                </div>
                <div className="file-details">
                  <span className="file-size">{formatFileSize(file.size)}</span>
                  {file.qualityScore && (
                    <span className={`quality-score ${
                      file.qualityScore >= 90 ? 'excellent' : 
                      file.qualityScore >= 70 ? 'good' : 'fair'
                    }`}>
                      Quality: {file.qualityScore}%
                    </span>
                  )}
                </div>
                {file.warnings && file.warnings.length > 0 && (
                  <div className="file-warnings">
                    {file.warnings.map((warning, i) => (
                      <span key={i} className="warning-text">⚠ {warning}</span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Batch Actions */}
      {files.length > 0 && (
        <div className="batch-actions">
          <button className="btn-secondary" onClick={() => console.log('Clear all')}>
            Clear All
          </button>
          <button 
            className="btn-primary" 
            disabled={files.length === 0}
            onClick={() => console.log('Process all')}
          >
            Process All ({files.length})
          </button>
        </div>
      )}
    </div>
  )
}

export default FilePanel