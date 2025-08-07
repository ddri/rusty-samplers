import React, { useState } from 'react'
import { Settings, Play, ArrowRight, Zap, Target, Gauge } from 'lucide-react'
import { PluginInfo, FileInfo, ConversionSettings } from '../types'

interface ControlPanelProps {
  plugins: PluginInfo[]
  selectedFile: FileInfo | null
  isLoading: boolean
  onConversion: (outputFormat: string) => void
}

const ControlPanel: React.FC<ControlPanelProps> = ({
  plugins,
  selectedFile,
  isLoading,
  onConversion
}) => {
  const [settings, setSettings] = useState<ConversionSettings>({
    outputFormat: 'SFZ',
    outputPath: '',
    quality: 'balanced',
    preserveEnvelopes: true,
    preserveFilters: true,
    preserveLFOs: true
  })

  const [showAdvanced, setShowAdvanced] = useState(false)

  const outputFormats = ['SFZ', 'WAV', 'SF2'] // Based on available plugins

  const handleSettingChange = (key: keyof ConversionSettings, value: any) => {
    setSettings(prev => ({
      ...prev,
      [key]: value
    }))
  }

  const getQualityIcon = (quality: string) => {
    switch (quality) {
      case 'fast':
        return <Zap size={16} />
      case 'maximum':
        return <Target size={16} />
      default:
        return <Gauge size={16} />
    }
  }

  return (
    <div className="control-panel-container">
      <div className="panel-header">
        <h2>Conversion Settings</h2>
        {selectedFile && (
          <span className="selected-file">
            {selectedFile.name}
          </span>
        )}
      </div>

      {!selectedFile ? (
        <div className="no-selection">
          <Settings size={48} className="no-selection-icon" />
          <p>Select a file to configure conversion</p>
        </div>
      ) : (
        <>
          {/* Plugin Selection */}
          <div className="setting-section">
            <h3>Format Conversion</h3>
            <div className="format-flow">
              <div className="input-format">
                <span className="format-badge input">
                  {selectedFile.format || 'AKP'}
                </span>
              </div>
              <ArrowRight className="flow-arrow" size={20} />
              <div className="output-format">
                <select 
                  value={settings.outputFormat}
                  onChange={(e) => handleSettingChange('outputFormat', e.target.value)}
                  className="format-select"
                >
                  {outputFormats.map(format => (
                    <option key={format} value={format}>{format}</option>
                  ))}
                </select>
              </div>
            </div>
          </div>

          {/* Available Plugins */}
          <div className="setting-section">
            <h3>Available Plugins</h3>
            <div className="plugin-list">
              {plugins.map(plugin => (
                <div key={plugin.name} className="plugin-item">
                  <span className="plugin-name">{plugin.name}</span>
                  <div className="plugin-badges">
                    {plugin.extensions.map(ext => (
                      <span key={ext} className="extension-badge">{ext}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Quality Settings */}
          <div className="setting-section">
            <h3>Quality Settings</h3>
            <div className="quality-options">
              {(['fast', 'balanced', 'maximum'] as const).map(quality => (
                <label key={quality} className="quality-option">
                  <input
                    type="radio"
                    name="quality"
                    value={quality}
                    checked={settings.quality === quality}
                    onChange={(e) => handleSettingChange('quality', e.target.value)}
                  />
                  <div className="quality-label">
                    {getQualityIcon(quality)}
                    <span className="quality-name">
                      {quality.charAt(0).toUpperCase() + quality.slice(1)}
                    </span>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Advanced Settings */}
          <div className="setting-section">
            <h3 
              className="collapsible-header"
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              Advanced Settings
              <span className={`chevron ${showAdvanced ? 'open' : ''}`}>▼</span>
            </h3>
            {showAdvanced && (
              <div className="advanced-settings">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={settings.preserveEnvelopes}
                    onChange={(e) => handleSettingChange('preserveEnvelopes', e.target.checked)}
                  />
                  Preserve envelope parameters
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={settings.preserveFilters}
                    onChange={(e) => handleSettingChange('preserveFilters', e.target.checked)}
                  />
                  Preserve filter settings
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={settings.preserveLFOs}
                    onChange={(e) => handleSettingChange('preserveLFOs', e.target.checked)}
                  />
                  Preserve LFO configurations
                </label>
              </div>
            )}
          </div>

          {/* Conversion Action */}
          <div className="conversion-action">
            <button
              className="btn-primary conversion-btn"
              disabled={isLoading}
              onClick={() => onConversion(settings.outputFormat)}
            >
              {isLoading ? (
                <>
                  <div className="spinner" />
                  Converting...
                </>
              ) : (
                <>
                  <Play size={16} />
                  Convert to {settings.outputFormat}
                </>
              )}
            </button>
          </div>
        </>
      )}
    </div>
  )
}

export default ControlPanel