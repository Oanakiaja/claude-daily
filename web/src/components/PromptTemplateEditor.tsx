import { useState, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'

interface VariableInfo {
  name: string
  description: string
}

interface PromptTemplateEditorProps {
  title: string
  description: string
  currentValue: string | null
  defaultValue: string
  availableVariables: VariableInfo[]
  onSave: (value: string | null) => Promise<void>
  disabled?: boolean
}

export function PromptTemplateEditor({
  title,
  description,
  currentValue,
  defaultValue,
  availableVariables,
  onSave,
  disabled,
}: PromptTemplateEditorProps) {
  const [expanded, setExpanded] = useState(false)
  const [value, setValue] = useState(currentValue || '')
  const [isUsingDefault, setIsUsingDefault] = useState(!currentValue)
  const [saving, setSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const insertVariable = (varName: string) => {
    if (textareaRef.current && !isUsingDefault) {
      const start = textareaRef.current.selectionStart
      const end = textareaRef.current.selectionEnd
      const text = value
      const placeholder = `{{${varName}}}`
      const newValue = text.substring(0, start) + placeholder + text.substring(end)
      setValue(newValue)
      // Restore focus and cursor position
      setTimeout(() => {
        textareaRef.current?.focus()
        const newPos = start + placeholder.length
        textareaRef.current?.setSelectionRange(newPos, newPos)
      }, 0)
    }
  }

  const handleSave = async () => {
    setSaving(true)
    setSaveMessage(null)
    try {
      await onSave(isUsingDefault ? null : value)
      setSaveMessage('Saved')
      setTimeout(() => setSaveMessage(null), 2000)
    } catch {
      setSaveMessage('Failed to save')
    } finally {
      setSaving(false)
    }
  }

  const handleCopyDefault = () => {
    setValue(defaultValue)
  }

  const handleReset = () => {
    setIsUsingDefault(true)
    setValue('')
  }

  return (
    <div className="border border-orange-500/20 rounded-lg overflow-hidden">
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-3 flex items-center justify-between bg-daily-card hover:bg-daily-light/50 transition-colors"
      >
        <div className="text-left">
          <h3 className="text-lg font-medium text-orange-400">{title}</h3>
          <p className="text-sm text-gray-500">{description}</p>
        </div>
        <div className="flex items-center gap-2">
          {currentValue && (
            <span className="text-xs px-2 py-1 bg-orange-500/20 text-orange-400 rounded">
              Custom
            </span>
          )}
          <svg
            className={`w-5 h-5 text-gray-400 transition-transform ${expanded ? 'rotate-180' : ''}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </div>
      </button>

      {/* Expanded Content */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4 space-y-4 bg-daily-card">
              {/* Variables Reference */}
              <div className="bg-daily-dark/50 rounded-lg p-3">
                <h4 className="text-sm font-medium text-gray-400 mb-2">Available Variables</h4>
                <div className="flex flex-wrap gap-2">
                  {availableVariables.map((v) => (
                    <button
                      key={v.name}
                      onClick={() => insertVariable(v.name)}
                      disabled={isUsingDefault || disabled}
                      className="px-2 py-1 bg-orange-500/10 text-orange-400 rounded text-xs
                                 hover:bg-orange-500/20 transition-colors disabled:opacity-50
                                 disabled:cursor-not-allowed"
                      title={v.description}
                    >
                      {`{{${v.name}}}`}
                    </button>
                  ))}
                </div>
              </div>

              {/* Toggle: Use Default / Custom */}
              <div className="flex items-center gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name={`template-mode-${title}`}
                    checked={isUsingDefault}
                    onChange={() => setIsUsingDefault(true)}
                    disabled={disabled}
                    className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-200">Use Default Template</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name={`template-mode-${title}`}
                    checked={!isUsingDefault}
                    onChange={() => {
                      setIsUsingDefault(false)
                      if (!value) {
                        setValue(defaultValue)
                      }
                    }}
                    disabled={disabled}
                    className="w-4 h-4 text-orange-500 bg-daily-dark border-gray-600 focus:ring-orange-500"
                  />
                  <span className="text-gray-200">Custom Template</span>
                </label>
              </div>

              {/* Template Editor */}
              <textarea
                ref={textareaRef}
                value={isUsingDefault ? defaultValue : value}
                onChange={(e) => setValue(e.target.value)}
                disabled={disabled || isUsingDefault}
                className="w-full h-64 bg-daily-dark border border-gray-600 rounded-lg
                           p-4 text-gray-200 font-mono text-sm resize-y
                           focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none
                           disabled:opacity-60 disabled:cursor-not-allowed"
                placeholder="Enter custom template..."
              />

              {/* Actions */}
              <div className="flex items-center justify-between">
                <div className="flex gap-2">
                  {!isUsingDefault && (
                    <>
                      <button
                        onClick={handleCopyDefault}
                        disabled={disabled}
                        className="px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors"
                      >
                        Copy Default
                      </button>
                      <button
                        onClick={handleReset}
                        disabled={disabled}
                        className="px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors"
                      >
                        Reset to Default
                      </button>
                    </>
                  )}
                </div>
                <div className="flex items-center gap-3">
                  {saveMessage && (
                    <span
                      className={`text-sm ${
                        saveMessage === 'Saved' ? 'text-green-400' : 'text-red-400'
                      }`}
                    >
                      {saveMessage}
                    </span>
                  )}
                  <button
                    onClick={handleSave}
                    disabled={disabled || saving}
                    className="px-4 py-2 bg-orange-500 text-white rounded-lg
                               hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed
                               transition-colors"
                  >
                    {saving ? 'Saving...' : 'Save'}
                  </button>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
