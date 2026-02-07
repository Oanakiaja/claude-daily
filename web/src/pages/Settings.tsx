import { useState, useEffect, useCallback, useRef } from 'react'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { Config, DefaultTemplates } from '../hooks/useApi'
import { TemplateEditor } from '../components/TemplateEditor'
import { EXAMPLE_DATA } from '../data/templateExamples'
import { cn } from '../lib/utils'
import { useLanguage } from '../contexts/LanguageContext'

// Real data state for template preview
interface RealPreviewData {
  session_summary: Record<string, string> | null
  daily_summary: Record<string, string> | null
  skill_extract: Record<string, string> | null
  command_extract: Record<string, string> | null
}

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
  const [activeSection, setActiveSection] = useState('language')
  const [realPreviewData, setRealPreviewData] = useState<RealPreviewData>({
    session_summary: null,
    daily_summary: null,
    skill_extract: null,
    command_extract: null,
  })
  const contentRef = useRef<HTMLDivElement>(null)
  const { fetchConfig, updateConfig, fetchDefaultTemplates, fetchDates, fetchSessions, fetchSession, fetchDailySummary, error } = useApi()
  const { t } = useLanguage()

  const NAV_SECTIONS = {
    general: [
      { id: 'language', label: t('settings.nav.language'), icon: '🌐' },
      { id: 'model', label: t('settings.nav.model'), icon: '🤖' },
      { id: 'features', label: t('settings.nav.features'), icon: '⚙️' },
      { id: 'digest-time', label: t('settings.nav.digestTime'), icon: '⏰' },
      { id: 'author', label: t('settings.nav.author'), icon: '✍️' },
      { id: 'info', label: t('settings.nav.info'), icon: '💾' },
    ],
    templates: [
      { id: 'session-template', label: t('settings.nav.sessionTemplate'), icon: '📝' },
      { id: 'daily-template', label: t('settings.nav.dailyTemplate'), icon: '📅' },
      { id: 'skill-template', label: t('settings.nav.skillTemplate'), icon: '🎯' },
      { id: 'command-template', label: t('settings.nav.commandTemplate'), icon: '⌨️' },
    ],
  }

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

  // Load real archive data for template preview
  const loadRealPreviewData = useCallback(async () => {
    try {
      // Get all dates
      const dates = await fetchDates()
      if (!dates || dates.length === 0) return

      // Get the most recent date
      const recentDate = dates[0].date

      // Get sessions for that date
      const sessions = await fetchSessions(recentDate)

      // Try to get session detail for session_summary preview
      if (sessions && sessions.length > 0) {
        const sessionDetail = await fetchSession(recentDate, sessions[0].name)
        if (sessionDetail) {
          const sessionData: Record<string, string> = {
            transcript: sessionDetail.content || '',
            cwd: sessionDetail.metadata?.cwd || '',
            git_branch: sessionDetail.metadata?.git_branch || '',
            language: config?.summary_language || 'en',
          }
          setRealPreviewData(prev => ({
            ...prev,
            session_summary: sessionData,
            // Also use session content for skill/command extract
            skill_extract: {
              session_content: sessionDetail.content || '',
              skill_hint: '',
              today: recentDate,
              language: config?.summary_language || 'en',
            },
            command_extract: {
              session_content: sessionDetail.content || '',
              command_hint: '',
              language: config?.summary_language || 'en',
            },
          }))
        }
      }

      // Try to get daily summary for daily_summary preview
      const dailySummary = await fetchDailySummary(recentDate)
      if (dailySummary && (dailySummary.overview || dailySummary.insights)) {
        const now = new Date()
        const hours = now.getHours()
        const currentPeriod = hours < 12 ? 'morning' : hours < 18 ? 'afternoon' : 'evening'
        const currentTime = `${String(hours).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`

        const existingContent = [
          dailySummary.overview ? `## Overview\n\n${dailySummary.overview}` : '',
          dailySummary.insights ? `## Insights\n\n${dailySummary.insights}` : '',
          dailySummary.tomorrow_focus ? `## Tomorrow Focus\n\n${dailySummary.tomorrow_focus}` : '',
        ].filter(Boolean).join('\n\n')

        const dailyData: Record<string, string> = {
          date: recentDate,
          current_time: currentTime,
          current_period: currentPeriod,
          periods_desc: 'morning (before 12:00), afternoon (12:00-18:00), evening (after 18:00)',
          existing_section: existingContent,
          sessions_section: '',
          sessions_json: JSON.stringify(sessions?.map(s => ({
            title: s.title || s.name,
            summary: s.summary_preview || '',
          })) || [], null, 2),
          language: config?.summary_language || 'en',
        }
        setRealPreviewData(prev => ({ ...prev, daily_summary: dailyData }))
      }
    } catch (err) {
      console.error('Failed to load real preview data:', err)
    }
  }, [fetchDates, fetchSessions, fetchSession, fetchDailySummary, config?.summary_language])

  useEffect(() => {
    loadConfig()
    loadDefaultTemplates()
  }, [loadConfig, loadDefaultTemplates])

  // Load real preview data when entering template sections
  useEffect(() => {
    if (activeSection.includes('template') && config) {
      loadRealPreviewData()
    }
  }, [activeSection, config, loadRealPreviewData])

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
      setSaveMessage(t('settings.saved'))
      setTimeout(() => setSaveMessage(null), 2000)
    } catch (err) {
      console.error('Failed to save config:', err)
    } finally {
      setSaving(false)
    }
  }

  const scrollToSection = (id: string) => {
    setActiveSection(id)
    // Wait for DOM to update before scrolling
    setTimeout(() => {
      const element = document.getElementById(id)
      if (element && contentRef.current) {
        const container = contentRef.current
        const elementTop = element.offsetTop - container.offsetTop
        container.scrollTo({ top: elementTop - 20, behavior: 'smooth' })
      }
    }, 0)
  }

  if (!config) {
    return (
      <div className="flex h-[calc(100vh-4rem)] items-center justify-center">
        <div className="text-gray-500">{t('settings.loading')}</div>
      </div>
    )
  }

  return (
    <div className="flex h-[calc(100vh-4rem)] overflow-hidden">
      {/* Left Navigation */}
      <aside className="w-64 shrink-0 border-r border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-black overflow-y-auto transition-colors">
        <div className="p-4 border-b border-gray-200 dark:border-gray-800">
          <h2 className="text-lg font-semibold text-orange-500 dark:text-orange-400">{t('settings.title')}</h2>
          <p className="text-xs text-gray-500 mt-1">{t('settings.subtitle')}</p>
        </div>
        <nav className="p-3">
          {/* General Settings */}
          <div className="mb-4">
            <h3 className="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
              {t('settings.general')}
            </h3>
            <div className="space-y-1">
              {NAV_SECTIONS.general.map((item) => (
                <button
                  key={item.id}
                  onClick={() => scrollToSection(item.id)}
                  className={cn(
                    'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left text-sm transition-colors',
                    activeSection === item.id
                      ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400 border border-orange-500/30'
                      : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                  )}
                >
                  <span className="text-base">{item.icon}</span>
                  <span>{item.label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Template Settings */}
          <div>
            <h3 className="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
              {t('settings.templates')}
            </h3>
            <div className="space-y-1">
              {NAV_SECTIONS.templates.map((item) => (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={cn(
                    'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left text-sm transition-colors',
                    activeSection === item.id
                      ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400 border border-orange-500/30'
                      : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                  )}
                >
                  <span className="text-base">{item.icon}</span>
                  <span>{item.label}</span>
                </button>
              ))}
            </div>
          </div>
        </nav>
      </aside>

      {/* Right Content */}
      <main ref={contentRef} className="flex-1 overflow-hidden">
        {/* Template Editor View (Full Screen) */}
        {activeSection === 'session-template' && defaultTemplates && (
          <TemplateEditor
            title={t('template.sessionTitle')}
            description={t('template.sessionDesc')}
            currentValue={config.prompt_templates.session_summary}
            defaultValue={
              config.summary_language === 'zh'
                ? defaultTemplates.session_summary_zh
                : defaultTemplates.session_summary_en
            }
            availableVariables={TEMPLATE_VARIABLES.session_summary}
            exampleData={EXAMPLE_DATA.session_summary}
            realData={realPreviewData.session_summary}
            onSave={async (value) => {
              const updated = await updateConfig({
                prompt_templates: { session_summary: value },
              })
              setConfig(updated)
            }}
            disabled={saving}
          />
        )}

        {activeSection === 'daily-template' && defaultTemplates && (
          <TemplateEditor
            title={t('template.dailyTitle')}
            description={t('template.dailyDesc')}
            currentValue={config.prompt_templates.daily_summary}
            defaultValue={
              config.summary_language === 'zh'
                ? defaultTemplates.daily_summary_zh
                : defaultTemplates.daily_summary_en
            }
            availableVariables={TEMPLATE_VARIABLES.daily_summary}
            exampleData={EXAMPLE_DATA.daily_summary}
            realData={realPreviewData.daily_summary}
            onSave={async (value) => {
              const updated = await updateConfig({
                prompt_templates: { daily_summary: value },
              })
              setConfig(updated)
            }}
            disabled={saving}
          />
        )}

        {activeSection === 'skill-template' && defaultTemplates && (
          <TemplateEditor
            title={t('template.skillTitle')}
            description={t('template.skillDesc')}
            currentValue={config.prompt_templates.skill_extract}
            defaultValue={
              config.summary_language === 'zh'
                ? defaultTemplates.skill_extract_zh
                : defaultTemplates.skill_extract_en
            }
            availableVariables={TEMPLATE_VARIABLES.skill_extract}
            exampleData={EXAMPLE_DATA.skill_extract}
            realData={realPreviewData.skill_extract}
            onSave={async (value) => {
              const updated = await updateConfig({
                prompt_templates: { skill_extract: value },
              })
              setConfig(updated)
            }}
            disabled={saving}
          />
        )}

        {activeSection === 'command-template' && defaultTemplates && (
          <TemplateEditor
            title={t('template.commandTitle')}
            description={t('template.commandDesc')}
            currentValue={config.prompt_templates.command_extract}
            defaultValue={
              config.summary_language === 'zh'
                ? defaultTemplates.command_extract_zh
                : defaultTemplates.command_extract_en
            }
            availableVariables={TEMPLATE_VARIABLES.command_extract}
            exampleData={EXAMPLE_DATA.command_extract}
            realData={realPreviewData.command_extract}
            onSave={async (value) => {
              const updated = await updateConfig({
                prompt_templates: { command_extract: value },
              })
              setConfig(updated)
            }}
            disabled={saving}
          />
        )}

        {/* General Settings View (Scrollable) */}
        {!activeSection.includes('template') && (
          <div className="h-full overflow-y-auto">
            <div className="max-w-4xl mx-auto px-6 py-8">
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
            <section id="language" className="bg-gray-50 dark:bg-daily-light border border-gray-200 dark:border-orange-500/20 rounded-xl p-6 transition-colors">
              <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">{t('settings.language.title')}</h2>
              <p className="text-gray-500 dark:text-gray-400 text-sm mb-4">
                {t('settings.language.desc')}
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
                    className="w-4 h-4 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-700 dark:text-gray-200">{t('settings.language.english')}</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="summary_language"
                    value="zh"
                    checked={config.summary_language === 'zh'}
                    onChange={(e) => handleChange('summary_language', e.target.value)}
                    disabled={saving}
                    className="w-4 h-4 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-700 dark:text-gray-200">{t('settings.language.chinese')}</span>
                </label>
              </div>
            </section>

            {/* Model Selection */}
            <section id="model" className="bg-gray-50 dark:bg-daily-light border border-gray-200 dark:border-orange-500/20 rounded-xl p-6 transition-colors">
              <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">{t('settings.model.title')}</h2>
              <p className="text-gray-500 dark:text-gray-400 text-sm mb-4">
                {t('settings.model.desc')}
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
                    className="w-4 h-4 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-700 dark:text-gray-200">{t('settings.model.sonnet')}</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="model"
                    value="haiku"
                    checked={config.model === 'haiku'}
                    onChange={(e) => handleChange('model', e.target.value)}
                    disabled={saving}
                    className="w-4 h-4 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-700 dark:text-gray-200">{t('settings.model.haiku')}</span>
                </label>
              </div>
            </section>

            {/* Feature Toggles */}
            <section id="features" className="bg-gray-50 dark:bg-daily-light border border-gray-200 dark:border-orange-500/20 rounded-xl p-6 transition-colors">
              <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">{t('settings.features.title')}</h2>
              <div className="space-y-4">
                <label className="flex items-center justify-between cursor-pointer">
                  <div>
                    <span className="text-gray-700 dark:text-gray-200">{t('settings.features.dailySummary')}</span>
                    <p className="text-gray-500 text-sm">{t('settings.features.dailySummaryDesc')}</p>
                  </div>
                  <input
                    type="checkbox"
                    checked={config.enable_daily_summary}
                    onChange={(e) => handleChange('enable_daily_summary', e.target.checked)}
                    disabled={saving}
                    className="w-5 h-5 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 rounded focus:ring-orange-500"
                  />
                </label>

                <label className="flex items-center justify-between cursor-pointer">
                  <div>
                    <span className="text-gray-700 dark:text-gray-200">{t('settings.features.extractionHints')}</span>
                    <p className="text-gray-500 text-sm">{t('settings.features.extractionHintsDesc')}</p>
                  </div>
                  <input
                    type="checkbox"
                    checked={config.enable_extraction_hints}
                    onChange={(e) => handleChange('enable_extraction_hints', e.target.checked)}
                    disabled={saving}
                    className="w-5 h-5 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 rounded focus:ring-orange-500"
                  />
                </label>

                <label className="flex items-center justify-between cursor-pointer">
                  <div>
                    <span className="text-gray-700 dark:text-gray-200">{t('settings.features.autoDigest')}</span>
                    <p className="text-gray-500 text-sm">{t('settings.features.autoDigestDesc')}</p>
                  </div>
                  <input
                    type="checkbox"
                    checked={config.auto_digest_enabled}
                    onChange={(e) => handleChange('auto_digest_enabled', e.target.checked)}
                    disabled={saving}
                    className="w-5 h-5 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 rounded focus:ring-orange-500"
                  />
                </label>

                <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
                  <h3 className="text-sm font-medium text-gray-600 dark:text-gray-300 mb-3">{t('settings.features.autoSummarizeTitle')}</h3>
                  <p className="text-gray-500 text-xs mb-4">
                    {t('settings.features.autoSummarizeDesc')}
                  </p>

                  <label className="flex items-center justify-between cursor-pointer mb-4">
                    <div>
                      <span className="text-gray-700 dark:text-gray-200">{t('settings.features.enableAutoSummarize')}</span>
                      <p className="text-gray-500 text-sm">{t('settings.features.enableAutoSummarizeDesc')}</p>
                    </div>
                    <input
                      type="checkbox"
                      checked={config.auto_summarize_enabled}
                      onChange={(e) => handleChange('auto_summarize_enabled', e.target.checked)}
                      disabled={saving}
                      className="w-5 h-5 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 rounded focus:ring-orange-500"
                    />
                  </label>

                  <label className="flex items-center justify-between cursor-pointer mb-4">
                    <div>
                      <span className="text-gray-700 dark:text-gray-200">{t('settings.features.triggerOnShow')}</span>
                      <p className="text-gray-500 text-sm">{t('settings.features.triggerOnShowDesc')}</p>
                    </div>
                    <input
                      type="checkbox"
                      checked={config.auto_summarize_on_show}
                      onChange={(e) => handleChange('auto_summarize_on_show', e.target.checked)}
                      disabled={saving || !config.auto_summarize_enabled}
                      className="w-5 h-5 text-orange-500 bg-white dark:bg-daily-dark border-gray-300 dark:border-gray-600 rounded focus:ring-orange-500 disabled:opacity-50"
                    />
                  </label>

                  <div className="flex items-center justify-between">
                    <div>
                      <span className="text-gray-700 dark:text-gray-200">{t('settings.features.inactiveThreshold')}</span>
                      <p className="text-gray-500 text-sm">{t('settings.features.inactiveThresholdDesc')}</p>
                    </div>
                    <input
                      type="number"
                      min="5"
                      max="480"
                      value={config.auto_summarize_inactive_minutes}
                      onChange={(e) => handleChange('auto_summarize_inactive_minutes', parseInt(e.target.value) || 30)}
                      disabled={saving || !config.auto_summarize_enabled}
                      className="w-20 bg-white dark:bg-daily-dark border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-1 text-gray-700 dark:text-gray-200 focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none disabled:opacity-50 text-center"
                    />
                  </div>
                </div>
              </div>
            </section>

            {/* Digest Time */}
            <section id="digest-time" className="bg-gray-50 dark:bg-daily-light border border-gray-200 dark:border-orange-500/20 rounded-xl p-6 transition-colors">
              <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">{t('settings.digestTime.title')}</h2>
              <p className="text-gray-500 dark:text-gray-400 text-sm mb-4">
                {t('settings.digestTime.desc')}
              </p>
              <input
                type="time"
                value={config.digest_time}
                onChange={(e) => handleChange('digest_time', e.target.value)}
                disabled={saving}
                className="bg-white dark:bg-daily-dark border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-2 text-gray-700 dark:text-gray-200 focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none"
              />
            </section>

            {/* Author */}
            <section id="author" className="bg-gray-50 dark:bg-daily-light border border-gray-200 dark:border-orange-500/20 rounded-xl p-6 transition-colors">
              <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">{t('settings.author.title')}</h2>
              <p className="text-gray-500 dark:text-gray-400 text-sm mb-4">
                {t('settings.author.desc')}
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
                placeholder={t('settings.author.placeholder')}
                className="w-full bg-white dark:bg-daily-dark border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-2 text-gray-700 dark:text-gray-200 placeholder-gray-400 dark:placeholder-gray-500 focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none"
              />
            </section>

            {/* Info Section (read-only) */}
            <section id="info" className="bg-gray-100 dark:bg-daily-dark/50 border border-gray-200 dark:border-gray-700 rounded-xl p-6 transition-colors">
              <h2 className="text-lg font-semibold text-gray-500 dark:text-gray-400 mb-3">{t('settings.info.title')}</h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-500">{t('settings.info.storagePath')}</span>
                  <span className="text-gray-700 dark:text-gray-300 font-mono text-xs">{config.storage_path}</span>
                </div>
              </div>
              <p className="text-gray-500 dark:text-gray-600 text-xs mt-4">
                {t('settings.info.cliHint')} <code className="bg-gray-200 dark:bg-gray-800 px-1 rounded">daily config --set-storage &lt;path&gt;</code>
              </p>
            </section>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}
