import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { format, parseISO, isToday } from 'date-fns'
import { useApi } from '../hooks/useApi'
import type { DateItem, DailySummary, Job } from '../hooks/useApi'
import { cn } from '../lib/utils'

interface DayCardData extends DateItem {
  summary?: DailySummary
}

export function Welcome() {
  const [days, setDays] = useState<DayCardData[]>([])
  const [loading, setLoading] = useState(true)
  const [autoSummarizeJobs, setAutoSummarizeJobs] = useState<Job[]>([])
  const { fetchDates, fetchDailySummary, fetchJobs } = useApi()
  const navigate = useNavigate()

  useEffect(() => {
    const loadData = async () => {
      try {
        const dates = await fetchDates()
        setDays(dates.map(d => ({ ...d })))

        // Load summaries for dates that have digest
        const summaryPromises = dates
          .filter(d => d.has_digest)
          .map(async (d) => {
            try {
              const summary = await fetchDailySummary(d.date)
              return { date: d.date, summary }
            } catch {
              return { date: d.date, summary: undefined }
            }
          })

        const summaries = await Promise.all(summaryPromises)
        const summaryMap = new Map(summaries.map(s => [s.date, s.summary]))

        setDays(dates.map(d => ({
          ...d,
          summary: summaryMap.get(d.date)
        })))
      } catch (err) {
        console.error('Failed to load data:', err)
      } finally {
        setLoading(false)
      }
    }

    loadData()
  }, [fetchDates, fetchDailySummary])

  // Poll for auto-summarize jobs
  useEffect(() => {
    const loadJobs = async () => {
      try {
        const jobs = await fetchJobs()
        const runningAutoJobs = jobs.filter(
          (j) => j.job_type === 'auto_summarize' && j.status_type === 'running'
        )
        setAutoSummarizeJobs(runningAutoJobs)
      } catch {
        // Silently ignore job fetch errors
      }
    }

    loadJobs()
    const interval = setInterval(loadJobs, 3000)
    return () => clearInterval(interval)
  }, [fetchJobs])

  const getWeekday = (dateStr: string) => {
    return format(parseISO(dateStr), 'EEEE')
  }

  const truncateText = (text: string, maxLength: number = 150) => {
    if (text.length <= maxLength) return text
    return text.slice(0, maxLength).trim() + '...'
  }

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto px-6 py-8">
        <h1 className="text-3xl font-bold mb-8">
          <span className="text-orange-500 dark:text-orange-400">Daily</span> Archives
        </h1>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[...Array(6)].map((_, i) => (
            <div key={i} className="h-48 bg-gray-200 dark:bg-daily-light rounded-xl animate-pulse" />
          ))}
        </div>
      </div>
    )
  }

  if (days.length === 0) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="flex flex-col items-center justify-center min-h-[60vh]">
          <div className="text-center space-y-4">
            <h1 className="text-4xl font-bold text-balance mb-2">
              Welcome to <span className="text-orange-500 dark:text-orange-400">Daily</span>
            </h1>
            <p className="text-gray-500 dark:text-gray-400 text-lg max-w-md mx-auto">
              Your context archive system for Claude Code sessions
            </p>
            <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-800">
              <p className="text-gray-500 text-sm">
                No archives found. Start a Claude Code session to begin archiving.
              </p>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-6xl mx-auto px-6 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">
          <span className="text-orange-500 dark:text-orange-400">Daily</span> Archives
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          {days.length} {days.length === 1 ? 'day' : 'days'} of Claude Code sessions
        </p>
      </div>

      {/* Auto-summarize notification */}
      <AnimatePresence>
        {autoSummarizeJobs.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="mb-6 p-4 rounded-lg border border-blue-500/30 bg-blue-500/10"
          >
            <div className="flex items-center gap-3">
              <span className="text-blue-400 text-lg">🤖</span>
              <div className="flex-1">
                <p className="text-blue-400 font-medium">
                  Auto-summarizing {autoSummarizeJobs.length} missed session{autoSummarizeJobs.length > 1 ? 's' : ''}
                </p>
                <p className="text-blue-400/70 text-sm mt-1">
                  Sessions without session_end hook are being summarized automatically
                </p>
              </div>
              <div className="size-2 bg-blue-400 rounded-full animate-pulse" />
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {days.map((day) => (
          <motion.div
            key={day.date}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.2 }}
          >
            <button
              onClick={() => navigate(`/day/${day.date}`)}
              className={cn(
                'w-full h-44 text-left p-5 rounded-xl border transition-all duration-200',
                'bg-gray-50 dark:bg-daily-light',
                'border-gray-200 dark:border-gray-800',
                'hover:border-orange-500/50 hover:shadow-lg hover:shadow-orange-500/10',
                'hover:scale-[1.02] active:scale-[0.98]',
                'group flex flex-col'
              )}
            >
              {/* Header */}
              <div className="flex items-start justify-between mb-3">
                <div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-lg font-semibold tabular-nums">{day.date}</span>
                    {isToday(parseISO(day.date)) && (
                      <span className="px-2 py-0.5 text-xs font-medium bg-orange-500/20 text-orange-500 dark:text-orange-400 rounded-full">
                        Today
                      </span>
                    )}
                  </div>
                  <span className="text-sm text-gray-500">{getWeekday(day.date)}</span>
                </div>
                <div className="text-right">
                  <span className="text-2xl font-bold text-orange-500 dark:text-orange-400">
                    {day.session_count}
                  </span>
                  <p className="text-xs text-gray-500">
                    {day.session_count === 1 ? 'session' : 'sessions'}
                  </p>
                </div>
              </div>

              {/* Summary Preview */}
              <div className="flex-1 mt-3 pt-3 border-t border-gray-200 dark:border-gray-700 overflow-hidden">
                {day.summary?.overview ? (
                  <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                    {truncateText(day.summary.overview, 100)}
                  </p>
                ) : (
                  <p className="text-sm text-gray-400 dark:text-gray-600 italic">
                    No summary yet
                  </p>
                )}
              </div>
            </button>
          </motion.div>
        ))}
      </div>
    </div>
  )
}
