import React from 'react'
import { BarChart, CheckCircle, AlertTriangle, XCircle, TrendingUp } from 'lucide-react'
import { FileInfo, QualityPreview } from '../types'

interface QualityPanelProps {
  selectedFile: FileInfo | null
  qualityPreview: QualityPreview | null
  isLoading: boolean
}

const QualityPanel: React.FC<QualityPanelProps> = ({
  selectedFile,
  qualityPreview,
  isLoading
}) => {
  const getQualityColor = (score: number) => {
    if (score >= 90) return 'excellent'
    if (score >= 70) return 'good'
    if (score >= 50) return 'fair'
    return 'poor'
  }

  const getQualityIcon = (score: number) => {
    if (score >= 90) return <CheckCircle className="quality-icon excellent" size={24} />
    if (score >= 70) return <TrendingUp className="quality-icon good" size={24} />
    if (score >= 50) return <AlertTriangle className="quality-icon fair" size={24} />
    return <XCircle className="quality-icon poor" size={24} />
  }

  return (
    <div className="quality-panel-container">
      <div className="panel-header">
        <h2>Quality Preview</h2>
        {selectedFile && (
          <span className="analysis-status">
            {isLoading ? 'Analyzing...' : 'Ready'}
          </span>
        )}
      </div>

      {!selectedFile ? (
        <div className="no-selection">
          <BarChart size={48} className="no-selection-icon" />
          <p>Select a file to see quality preview</p>
          <p className="no-selection-subtext">
            Real-time parameter preservation analysis
          </p>
        </div>
      ) : isLoading ? (
        <div className="loading-state">
          <div className="spinner-large" />
          <p>Analyzing conversion quality...</p>
        </div>
      ) : qualityPreview ? (
        <>
          {/* Overall Quality Score */}
          <div className="quality-score-section">
            <div className="quality-header">
              {getQualityIcon(qualityPreview.score)}
              <div className="quality-info">
                <h3>Conversion Quality</h3>
                <div className={`quality-score-large ${getQualityColor(qualityPreview.score)}`}>
                  {qualityPreview.score}%
                </div>
              </div>
            </div>
            
            <div className="quality-bar">
              <div 
                className={`quality-fill ${getQualityColor(qualityPreview.score)}`}
                style={{ width: `${qualityPreview.score}%` }}
              />
            </div>
          </div>

          {/* Parameter Breakdown */}
          <div className="parameter-section">
            <h3>Parameter Preservation</h3>
            <div className="parameter-stats">
              <div className="stat-item">
                <div className="stat-value preserved">
                  {qualityPreview.parameters_preserved}
                </div>
                <div className="stat-label">Preserved</div>
              </div>
              <div className="stat-item">
                <div className="stat-value lost">
                  {qualityPreview.parameters_lost}
                </div>
                <div className="stat-label">Lost</div>
              </div>
            </div>

            {/* Parameter Categories */}
            <div className="parameter-categories">
              <div className="category-item">
                <CheckCircle size={16} className="category-icon success" />
                <span className="category-name">Envelopes</span>
                <span className="category-status">✓ Fully preserved</span>
              </div>
              <div className="category-item">
                <CheckCircle size={16} className="category-icon success" />
                <span className="category-name">Filters</span>
                <span className="category-status">✓ Fully preserved</span>
              </div>
              <div className="category-item">
                <AlertTriangle size={16} className="category-icon warning" />
                <span className="category-name">LFO</span>
                <span className="category-status">⚠ Approximated</span>
              </div>
              <div className="category-item">
                <XCircle size={16} className="category-icon error" />
                <span className="category-name">Mod Matrix</span>
                <span className="category-status">✗ Not supported</span>
              </div>
            </div>
          </div>

          {/* Warnings & Recommendations */}
          {qualityPreview.warnings.length > 0 && (
            <div className="warnings-section">
              <h3>Conversion Warnings</h3>
              <div className="warning-list">
                {qualityPreview.warnings.map((warning, index) => (
                  <div key={index} className="warning-item">
                    <AlertTriangle size={16} className="warning-icon" />
                    <span className="warning-text">{warning}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recommendations */}
          <div className="recommendations-section">
            <h3>Recommendations</h3>
            <div className="recommendation-list">
              {qualityPreview.score < 90 && (
                <div className="recommendation-item">
                  <TrendingUp size={16} className="rec-icon" />
                  <span className="rec-text">
                    Enable "Maximum Quality" mode to improve parameter preservation
                  </span>
                </div>
              )}
              <div className="recommendation-item">
                <CheckCircle size={16} className="rec-icon" />
                <span className="rec-text">
                  SFZ format recommended for best parameter compatibility
                </span>
              </div>
            </div>
          </div>

          {/* Technical Details (Collapsible) */}
          <div className="technical-details">
            <details>
              <summary>Technical Details</summary>
              <div className="technical-content">
                <div className="tech-row">
                  <span className="tech-label">Source Format:</span>
                  <span className="tech-value">{selectedFile.format}</span>
                </div>
                <div className="tech-row">
                  <span className="tech-label">Keygroups:</span>
                  <span className="tech-value">4 detected</span>
                </div>
                <div className="tech-row">
                  <span className="tech-label">Sample Rate:</span>
                  <span className="tech-value">44.1 kHz</span>
                </div>
                <div className="tech-row">
                  <span className="tech-label">Bit Depth:</span>
                  <span className="tech-value">16-bit</span>
                </div>
              </div>
            </details>
          </div>
        </>
      ) : (
        <div className="error-state">
          <AlertTriangle size={48} className="error-icon" />
          <p>Failed to analyze file quality</p>
          <p className="error-subtext">
            Please try selecting the file again
          </p>
        </div>
      )}
    </div>
  )
}

export default QualityPanel