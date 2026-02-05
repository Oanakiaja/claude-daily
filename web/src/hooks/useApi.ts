import { useState, useCallback } from 'react'

const API_BASE = '/api'

export interface DateItem {
  date: string
  session_count: number
  has_digest: boolean
}

export interface DailySummary {
  overview?: string
  insights?: string
  tomorrow_focus?: string
  file_path?: string
}

export interface Session {
  name: string
  title?: string
  summary_preview?: string
}

export interface SessionDetail {
  content: string
  metadata?: {
    title?: string
    cwd?: string
    git_branch?: string
    duration?: string
  }
  file_path?: string
}

export interface Job {
  id: string
  task_name: string
  status: string
  status_type: 'running' | 'completed' | 'failed'
  job_type: 'session_end' | 'auto_summarize' | 'manual'
  started_at: string
  elapsed: string
}

export interface DigestResponse {
  message: string
  session_count: number
}

export interface PromptTemplates {
  session_summary: string | null
  daily_summary: string | null
  skill_extract: string | null
  command_extract: string | null
}

export interface PromptTemplatesUpdate {
  session_summary?: string | null
  daily_summary?: string | null
  skill_extract?: string | null
  command_extract?: string | null
}

export interface DefaultTemplates {
  session_summary_en: string
  session_summary_zh: string
  daily_summary_en: string
  daily_summary_zh: string
  skill_extract_en: string
  skill_extract_zh: string
  command_extract_en: string
  command_extract_zh: string
}

export interface Config {
  storage_path: string
  model: string
  summary_language: string
  enable_daily_summary: boolean
  enable_extraction_hints: boolean
  auto_digest_enabled: boolean
  digest_time: string
  author: string | null
  prompt_templates: PromptTemplates
  auto_summarize_enabled: boolean
  auto_summarize_on_show: boolean
  auto_summarize_inactive_minutes: number
}

export interface ConfigUpdate {
  summary_language?: string
  model?: string
  enable_daily_summary?: boolean
  enable_extraction_hints?: boolean
  auto_digest_enabled?: boolean
  digest_time?: string
  author?: string
  prompt_templates?: PromptTemplatesUpdate
  auto_summarize_enabled?: boolean
  auto_summarize_on_show?: boolean
  auto_summarize_inactive_minutes?: number
}

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

interface RequestOptions extends RequestInit {
  headers?: Record<string, string>
}

export function useApi() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const request = useCallback(async <T>(endpoint: string, options: RequestOptions = {}): Promise<T> => {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(`${API_BASE}${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      })
      const data: ApiResponse<T> = await res.json()
      if (!data.success) {
        throw new Error(data.error || 'Request failed')
      }
      return data.data as T
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error'
      setError(message)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const fetchDates = useCallback(() => request<DateItem[]>('/dates'), [request])

  const fetchDailySummary = useCallback(
    (date: string) => request<DailySummary>(`/dates/${date}`),
    [request]
  )

  const fetchSessions = useCallback(
    (date: string) => request<Session[]>(`/dates/${date}/sessions`),
    [request]
  )

  const fetchSession = useCallback(
    (date: string, name: string) => request<SessionDetail>(`/dates/${date}/sessions/${encodeURIComponent(name)}`),
    [request]
  )

  const fetchJobs = useCallback(() => request<Job[]>('/jobs'), [request])

  const fetchJob = useCallback(
    (id: string) => request<Job>(`/jobs/${id}`),
    [request]
  )

  const fetchJobLog = useCallback(
    (id: string) => request<string>(`/jobs/${id}/log`),
    [request]
  )

  const killJob = useCallback(
    (id: string) => request<void>(`/jobs/${id}/kill`, { method: 'POST' }),
    [request]
  )

  const triggerDigest = useCallback(
    (date: string) => request<DigestResponse>(`/dates/${date}/digest`, { method: 'POST' }),
    [request]
  )

  const fetchConfig = useCallback(() => request<Config>('/config'), [request])

  const updateConfig = useCallback(
    (config: ConfigUpdate) =>
      request<Config>('/config', {
        method: 'PATCH',
        body: JSON.stringify(config),
      }),
    [request]
  )

  const fetchDefaultTemplates = useCallback(
    () => request<DefaultTemplates>('/config/templates/defaults'),
    [request]
  )

  return {
    loading,
    error,
    fetchDates,
    fetchDailySummary,
    fetchSessions,
    fetchSession,
    fetchJobs,
    fetchJob,
    fetchJobLog,
    killJob,
    triggerDigest,
    fetchConfig,
    updateConfig,
    fetchDefaultTemplates,
  }
}
