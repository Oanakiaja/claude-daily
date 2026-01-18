import { useState, useEffect, useCallback } from 'react'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { Config } from '../hooks/useApi'

export function Settings() {
  const [config, setConfig] = useState<Config | null>(null)
  const [saving, setSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const { fetchConfig, updateConfig, error } = useApi()

  const loadConfig = useCallback(() => {
    fetchConfig()
      .then(setConfig)
      .catch(console.error)
  }, [fetchConfig])

  useEffect(() => {
    loadConfig()
  }, [loadConfig])

  const handleChange = async (field: string, value: string | boolean) => {
    if (!config) return

    setSaving(true)
    setSaveMessage(null)

    try {
      const updated = await updateConfig({ [field]: value })
      setConfig(updated)
      setSaveMessage('Settings saved')
      setTimeout(() => setSaveMessage(null), 2000)
    } catch (err) {
      console.error('Failed to save config:', err)
    } finally {
      setSaving(false)
    }
  }

  if (!config) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="text-gray-500">Loading...</div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-balance">Settings</h1>
        <p className="text-gray-500 mt-2">Configure Daily summarization options</p>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 mb-6">
          {error}
        </div>
      )}

      {saveMessage && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          className="bg-green-500/10 border border-green-500/30 rounded-lg p-4 text-green-400 mb-6"
        >
          {saveMessage}
        </motion.div>
      )}

      <div className="space-y-6">
        {/* Summary Language */}
        <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
          <h2 className="text-xl font-semibold text-orange-400 mb-4">Summary Language</h2>
          <p className="text-gray-400 text-sm mb-4">
            Choose the language for AI-generated summaries and digests
          </p>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="summary_language"
                value="en"
                checked={config.summary_language === 'en'}
                onChange={(e) => handleChange('summary_language', e.target.value)}
                disabled={saving}
                className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
              <span className="text-gray-200">English</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="summary_language"
                value="zh"
                checked={config.summary_language === 'zh'}
                onChange={(e) => handleChange('summary_language', e.target.value)}
                disabled={saving}
                className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
              <span className="text-gray-200">Chinese / 中文</span>
            </label>
          </div>
        </section>

        {/* Model Selection */}
        <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
          <h2 className="text-xl font-semibold text-orange-400 mb-4">Summarization Model</h2>
          <p className="text-gray-400 text-sm mb-4">
            Choose the Claude model for generating summaries
          </p>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="model"
                value="sonnet"
                checked={config.model === 'sonnet'}
                onChange={(e) => handleChange('model', e.target.value)}
                disabled={saving}
                className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
              <span className="text-gray-200">Sonnet (smarter)</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="model"
                value="haiku"
                checked={config.model === 'haiku'}
                onChange={(e) => handleChange('model', e.target.value)}
                disabled={saving}
                className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
              <span className="text-gray-200">Haiku (faster, cheaper)</span>
            </label>
          </div>
        </section>

        {/* Feature Toggles */}
        <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
          <h2 className="text-xl font-semibold text-orange-400 mb-4">Features</h2>
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer">
              <div>
                <span className="text-gray-200">Enable Daily Summary</span>
                <p className="text-gray-500 text-sm">Generate daily digest from session summaries</p>
              </div>
              <input
                type="checkbox"
                checked={config.enable_daily_summary}
                onChange={(e) => handleChange('enable_daily_summary', e.target.checked)}
                disabled={saving}
                className="w-5 h-5 text-orange-500 bg-daily-dark border-gray-600 rounded focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
            </label>

            <label className="flex items-center justify-between cursor-pointer">
              <div>
                <span className="text-gray-200">Enable Extraction Hints</span>
                <p className="text-gray-500 text-sm">Suggest potential skills and commands to extract</p>
              </div>
              <input
                type="checkbox"
                checked={config.enable_extraction_hints}
                onChange={(e) => handleChange('enable_extraction_hints', e.target.checked)}
                disabled={saving}
                className="w-5 h-5 text-orange-500 bg-daily-dark border-gray-600 rounded focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
            </label>

            <label className="flex items-center justify-between cursor-pointer">
              <div>
                <span className="text-gray-200">Auto Digest</span>
                <p className="text-gray-500 text-sm">Automatically digest previous day's sessions on session start</p>
              </div>
              <input
                type="checkbox"
                checked={config.auto_digest_enabled}
                onChange={(e) => handleChange('auto_digest_enabled', e.target.checked)}
                disabled={saving}
                className="w-5 h-5 text-orange-500 bg-daily-dark border-gray-600 rounded focus:ring-orange-500 focus:ring-offset-daily-dark"
              />
            </label>
          </div>
        </section>

        {/* Info Section */}
        <section className="bg-daily-dark/50 border border-gray-700 rounded-xl p-6">
          <h2 className="text-lg font-semibold text-gray-400 mb-3">Current Configuration</h2>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-500">Storage Path</span>
              <span className="text-gray-300 font-mono">{config.storage_path}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Digest Time</span>
              <span className="text-gray-300">{config.digest_time}</span>
            </div>
            {config.author && (
              <div className="flex justify-between">
                <span className="text-gray-500">Author</span>
                <span className="text-gray-300">{config.author}</span>
              </div>
            )}
          </div>
          <p className="text-gray-600 text-xs mt-4">
            Use <code className="bg-gray-800 px-1 rounded">daily config -i</code> for full configuration options
          </p>
        </section>
      </div>
    </div>
  )
}
