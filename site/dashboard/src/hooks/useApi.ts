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
    tool_calls?: number
  }
}

export interface Job {
  id: string
  task_name: string
  status: string
  status_type: 'running' | 'completed' | 'failed'
  started_at: string
  elapsed: string
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
  }
}
