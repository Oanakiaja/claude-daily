import { useState, useRef, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'

interface VariableInfo {
  name: string
  description: string
}

interface TemplateEditorProps {
  title: string
  description: string
  currentValue: string | null
  defaultValue: string
  availableVariables: VariableInfo[]
  exampleData: Record<string, string>
  realData?: Record<string, string> | null
  onSave: (value: string | null) => Promise<void>
  disabled?: boolean
}

export function TemplateEditor({
  title,
  description,
  currentValue,
  defaultValue,
  availableVariables,
  exampleData,
  realData,
  onSave,
  disabled,
}: TemplateEditorProps) {
  const [value, setValue] = useState(currentValue || defaultValue)
  const [isUsingDefault, setIsUsingDefault] = useState(!currentValue)
  const [saving, setSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const [preview, setPreview] = useState('')
  const [useRealData, setUseRealData] = useState(!!realData)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Auto-switch to real data when it becomes available
  useEffect(() => {
    if (realData && !useRealData) {
      setUseRealData(true)
    }
  }, [realData])

  // Generate preview with selected data source
  useEffect(() => {
    const template = isUsingDefault ? defaultValue : value
    let rendered = template

    // Choose data source based on toggle
    const dataSource = useRealData && realData ? realData : exampleData

    // Replace all variables with data
    Object.entries(dataSource).forEach(([key, val]) => {
      const regex = new RegExp(`\\{\\{${key}\\}\\}`, 'g')
      rendered = rendered.replace(regex, val)
    })

    setPreview(rendered)
  }, [value, isUsingDefault, defaultValue, exampleData, realData, useRealData])

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
      setSaveMessage('Saved successfully')
      setTimeout(() => setSaveMessage(null), 2000)
    } catch {
      setSaveMessage('Failed to save')
    } finally {
      setSaving(false)
    }
  }

  const handleReset = () => {
    setIsUsingDefault(true)
    setValue(defaultValue)
  }

  const handleUseCustom = () => {
    setIsUsingDefault(false)
    if (!value || value === defaultValue) {
      setValue(currentValue || defaultValue)
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-daily-light px-6 py-4 transition-colors">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400">{title}</h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{description}</p>
          </div>
          <div className="flex items-center gap-2">
            {!isUsingDefault && (
              <span className="text-xs px-2 py-1 bg-orange-500/20 text-orange-400 rounded">
                Custom
              </span>
            )}
            {saveMessage && (
              <span
                className={`text-sm ${
                  saveMessage.includes('success') ? 'text-green-400' : 'text-red-400'
                }`}
              >
                {saveMessage}
              </span>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-3 mt-4">
          <button
            onClick={() => setIsUsingDefault(true)}
            disabled={disabled}
            className={`px-4 py-2 rounded-lg text-sm transition-colors ${
              isUsingDefault
                ? 'bg-orange-500 text-white'
                : 'bg-gray-200 dark:bg-daily-dark text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
            }`}
          >
            Use Default
          </button>
          <button
            onClick={handleUseCustom}
            disabled={disabled}
            className={`px-4 py-2 rounded-lg text-sm transition-colors ${
              !isUsingDefault
                ? 'bg-orange-500 text-white'
                : 'bg-gray-200 dark:bg-daily-dark text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
            }`}
          >
            Custom
          </button>
          <div className="flex-1" />
          {!isUsingDefault && (
            <button
              onClick={handleReset}
              disabled={disabled || saving}
              className="px-4 py-2 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
            >
              Reset to Default
            </button>
          )}
          <button
            onClick={handleSave}
            disabled={disabled || saving}
            className="px-6 py-2 bg-orange-500 text-white rounded-lg
                       hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed
                       transition-colors"
          >
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>

      {/* Split View */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Editor */}
        <div className="w-1/2 flex flex-col border-r border-gray-200 dark:border-gray-800">
          {/* Variables */}
          <div className="flex-shrink-0 bg-gray-100 dark:bg-daily-dark/50 border-b border-gray-200 dark:border-gray-800 px-4 py-3 transition-colors">
            <h4 className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">Available Variables (click to insert)</h4>
            <div className="flex flex-wrap gap-2">
              {availableVariables.map((v) => (
                <button
                  key={v.name}
                  onClick={() => insertVariable(v.name)}
                  disabled={isUsingDefault || disabled}
                  className="px-2 py-1 bg-orange-500/10 text-orange-500 dark:text-orange-400 rounded text-xs
                             hover:bg-orange-500/20 transition-colors disabled:opacity-50
                             disabled:cursor-not-allowed"
                  title={v.description}
                >
                  {`{{${v.name}}}`}
                </button>
              ))}
            </div>
          </div>

          {/* Textarea */}
          <div className="flex-1 p-4 overflow-hidden bg-white dark:bg-transparent transition-colors">
            <textarea
              ref={textareaRef}
              value={isUsingDefault ? defaultValue : value}
              onChange={(e) => setValue(e.target.value)}
              disabled={disabled || isUsingDefault}
              className="w-full h-full bg-gray-50 dark:bg-daily-dark border border-gray-300 dark:border-gray-600 rounded-lg
                         p-4 text-gray-800 dark:text-gray-200 font-mono text-sm resize-none
                         focus:border-orange-500 focus:ring-1 focus:ring-orange-500 outline-none
                         disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
              placeholder="Enter custom template..."
            />
          </div>
        </div>

        {/* Right: Preview */}
        <div className="w-1/2 flex flex-col bg-gray-50 dark:bg-daily-dark/30 transition-colors">
          <div className="flex-shrink-0 bg-gray-100 dark:bg-daily-dark/50 border-b border-gray-200 dark:border-gray-800 px-4 py-3 transition-colors">
            <div className="flex items-center justify-between">
              <h4 className="text-xs font-medium text-gray-500 dark:text-gray-400">Preview</h4>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setUseRealData(false)}
                  className={`px-2 py-1 text-xs rounded transition-colors ${
                    !useRealData
                      ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400'
                      : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300'
                  }`}
                >
                  Example
                </button>
                <button
                  onClick={() => setUseRealData(true)}
                  disabled={!realData}
                  className={`px-2 py-1 text-xs rounded transition-colors ${
                    useRealData && realData
                      ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400'
                      : realData
                        ? 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300'
                        : 'text-gray-300 dark:text-gray-600 cursor-not-allowed'
                  }`}
                  title={!realData ? 'No real archive data available' : 'Use real archive data'}
                >
                  Real Data
                </button>
              </div>
            </div>
          </div>
          <div className="flex-1 p-6 overflow-y-auto">
            <div className="prose dark:prose-invert max-w-none
                          prose-headings:text-orange-500 dark:prose-headings:text-orange-400
                          prose-a:text-orange-500 dark:prose-a:text-orange-400 prose-a:no-underline hover:prose-a:underline
                          prose-code:text-orange-600 dark:prose-code:text-orange-300 prose-code:bg-gray-100 dark:prose-code:bg-daily-dark prose-code:px-1 prose-code:rounded
                          prose-pre:bg-gray-100 dark:prose-pre:bg-daily-dark prose-pre:border prose-pre:border-gray-200 dark:prose-pre:border-gray-700
                          prose-strong:text-orange-600 dark:prose-strong:text-orange-300
                          prose-ul:text-gray-700 dark:prose-ul:text-gray-300 prose-ol:text-gray-700 dark:prose-ol:text-gray-300
                          prose-li:text-gray-700 dark:prose-li:text-gray-300">
              <ReactMarkdown>{preview}</ReactMarkdown>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
