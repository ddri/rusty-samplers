import React, { useCallback } from 'react'
import { Upload, FileText, AlertTriangle, CheckCircle, Clock } from 'lucide-react'
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

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    
    const droppedFiles = Array.from(e.dataTransfer.files)
    const fileInfos: FileInfo[] = droppedFiles.map(file => ({
      path: file.name, // In a real app, this would be the full path
      name: file.name,
      size: file.size,
      format: file.name.split('.').pop()?.toUpperCase(),
      status: 'pending'
    }))
    
    onFilesAdded(fileInfos)
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