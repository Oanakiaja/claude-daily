import { useState, useEffect, useCallback } from 'react'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { Config, DefaultTemplates } from '../hooks/useApi'
import { PromptTemplateEditor } from '../components/PromptTemplateEditor'

// Variable definitions for each template type
const TEMPLATE_VARIABLES = {
  session_summary: [
    { name: 'transcript', description: 'The session transcript content' },
    { name: 'cwd', description: 'Current working directory' },
    { name: 'git_branch', description: 'Current git branch name' },
    { name: 'language', description: 'Output language (en/zh)' },
  ],
  daily_summary: [
    { name: 'date', description: 'The date being summarized' },
    { name: 'current_time', description: 'Current time (HH:MM)' },
    { name: 'current_period', description: 'Time period (morning/afternoon/evening)' },
    { name: 'periods_desc', description: 'Description of time periods' },
    { name: 'existing_section', description: 'Existing summary content (if any)' },
    { name: 'sessions_section', description: 'Sessions data section' },
    { name: 'sessions_json', description: 'Sessions in JSON format' },
    { name: 'language', description: 'Output language (en/zh)' },
  ],
  skill_extract: [
    { name: 'session_content', description: 'The session summary content' },
    { name: 'skill_hint', description: 'Hint about what skill to extract' },
    { name: 'today', description: "Today's date" },
    { name: 'language', description: 'Output language (en/zh)' },
  ],
  command_extract: [
    { name: 'session_content', description: 'The session summary content' },
    { name: 'command_hint', description: 'Hint about what command to extract' },
    { name: 'language', description: 'Output language (en/zh)' },
  ],
}

export function Settings() {
  const [config, setConfig] = useState<Config | null>(null)
  const [defaultTemplates, setDefaultTemplates] = useState<DefaultTemplates | null>(null)
  const [saving, setSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const [authorInput, setAuthorInput] = useState('')
  const { fetchConfig, updateConfig, fetchDefaultTemplates, error } = useApi()

  const loadConfig = useCallback(() => {
    fetchConfig()
      .then((cfg) => {
        setConfig(cfg)
        setAuthorInput(cfg.author || '')
      })
      .catch(console.error)
  }, [fetchConfig])

  const loadDefaultTemplates = useCallback(() => {
    fetchDefaultTemplates()
      .then(setDefaultTemplates)
      .catch(console.error)
  }, [fetchDefaultTemplates])

  useEffect(() => {
    loadConfig()
    loadDefaultTemplates()
  }, [loadConfig, loadDefaultTemplates])

  const handleChange = async (field: string, value: string | boolean) => {
    if (!config) return

    setSaving(true)
    setSaveMessage(null)

    try {
      const updated = await updateConfig({ [field]: value })
      setConfig(updated)
      if (field === 'author') {
        setAuthorInput(updated.author || '')
      }
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

        {/* Digest Time */}
        <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
          <h2 className="text-xl font-semibold text-orange-400 mb-4">Digest Time</h2>
          <p className="text-gray-400 text-sm mb-4">
            Time to auto-digest previous day's sessions (format: HH:MM)
          </p>
          <input
            type="time"
            value={config.digest_time}
            onChange={(e) => handleChange('digest_time', e.target.value)}
            disabled={saving}
            className="bg-daily-dark border border-gray-600 rounded-lg px-4 py-2 text-gray-200 focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none"
          />
        </section>

        {/* Author */}
        <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
          <h2 className="text-xl font-semibold text-orange-400 mb-4">Author</h2>
          <p className="text-gray-400 text-sm mb-4">
            Author name for archive metadata (optional)
          </p>
          <input
            type="text"
            value={authorInput}
            onChange={(e) => setAuthorInput(e.target.value)}
            onBlur={(e) => {
              if (e.target.value !== (config.author || '')) {
                handleChange('author', e.target.value)
              }
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.currentTarget.blur()
              }
            }}
            disabled={saving}
            placeholder="Enter author name..."
            className="w-full bg-daily-dark border border-gray-600 rounded-lg px-4 py-2 text-gray-200 placeholder-gray-500 focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none"
          />
        </section>

        {/* Prompt Templates */}
        {defaultTemplates && (
          <section className="bg-daily-card border border-orange-500/20 rounded-xl p-6">
            <h2 className="text-xl font-semibold text-orange-400 mb-2">Prompt Templates</h2>
            <p className="text-gray-400 text-sm mb-4">
              Customize the AI prompts used for summarization. Use {'{{variable}}'} syntax for dynamic values.
            </p>
            <div className="space-y-3">
              <PromptTemplateEditor
                title="Session Summary"
                description="Template for summarizing individual sessions"
                currentValue={config.prompt_templates.session_summary}
                defaultValue={
                  config.summary_language === 'zh'
                    ? defaultTemplates.session_summary_zh
                    : defaultTemplates.session_summary_en
                }
                availableVariables={TEMPLATE_VARIABLES.session_summary}
                onSave={async (value) => {
                  const updated = await updateConfig({
                    prompt_templates: { session_summary: value },
                  })
                  setConfig(updated)
                }}
                disabled={saving}
              />

              <PromptTemplateEditor
                title="Daily Summary"
                description="Template for generating daily digests"
                currentValue={config.prompt_templates.daily_summary}
                defaultValue={
                  config.summary_language === 'zh'
                    ? defaultTemplates.daily_summary_zh
                    : defaultTemplates.daily_summary_en
                }
                availableVariables={TEMPLATE_VARIABLES.daily_summary}
                onSave={async (value) => {
                  const updated = await updateConfig({
                    prompt_templates: { daily_summary: value },
                  })
                  setConfig(updated)
                }}
                disabled={saving}
              />

              <PromptTemplateEditor
                title="Skill Extraction"
                description="Template for extracting reusable skills from sessions"
                currentValue={config.prompt_templates.skill_extract}
                defaultValue={
                  config.summary_language === 'zh'
                    ? defaultTemplates.skill_extract_zh
                    : defaultTemplates.skill_extract_en
                }
                availableVariables={TEMPLATE_VARIABLES.skill_extract}
                onSave={async (value) => {
                  const updated = await updateConfig({
                    prompt_templates: { skill_extract: value },
                  })
                  setConfig(updated)
                }}
                disabled={saving}
              />

              <PromptTemplateEditor
                title="Command Extraction"
                description="Template for extracting slash commands from sessions"
                currentValue={config.prompt_templates.command_extract}
                defaultValue={
                  config.summary_language === 'zh'
                    ? defaultTemplates.command_extract_zh
                    : defaultTemplates.command_extract_en
                }
                availableVariables={TEMPLATE_VARIABLES.command_extract}
                onSave={async (value) => {
                  const updated = await updateConfig({
                    prompt_templates: { command_extract: value },
                  })
                  setConfig(updated)
                }}
                disabled={saving}
              />
            </div>
          </section>
        )}

        {/* Info Section (read-only) */}
        <section className="bg-daily-dark/50 border border-gray-700 rounded-xl p-6">
          <h2 className="text-lg font-semibold text-gray-400 mb-3">Read-only Info</h2>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-500">Storage Path</span>
              <span className="text-gray-300 font-mono text-xs">{config.storage_path}</span>
            </div>
          </div>
          <p className="text-gray-600 text-xs mt-4">
            Storage path can only be changed via CLI: <code className="bg-gray-800 px-1 rounded">daily config --set-storage &lt;path&gt;</code>
          </p>
        </section>
      </div>
    </div>
  )
}
