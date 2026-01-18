import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { DailySummary, Session } from '../hooks/useApi'
import { SessionCard } from '../components/SessionCard'
import { MarkdownRenderer } from '../components/MarkdownRenderer'

export function DayDetail() {
  const { date } = useParams<{ date: string }>()
  const [summary, setSummary] = useState<DailySummary | null>(null)
  const [sessions, setSessions] = useState<Session[]>([])
  const { fetchDailySummary, fetchSessions, loading, error } = useApi()

  useEffect(() => {
    if (!date) return
    Promise.all([
      fetchDailySummary(date).then(setSummary),
      fetchSessions(date).then(setSessions),
    ]).catch(console.error)
  }, [date, fetchDailySummary, fetchSessions])

  if (loading && !summary) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="animate-pulse space-y-4">
          <div className="h-8 w-48 bg-daily-light rounded" />
          <div className="h-32 bg-daily-light rounded-lg" />
          <div className="h-24 bg-daily-light rounded-lg" />
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      {/* Breadcrumb */}
      <nav className="mb-6 text-sm">
        <Link to="/" className="text-gray-500 hover:text-gray-300">
          Archives
        </Link>
        <span className="text-gray-600 mx-2">/</span>
        <span className="text-orange-400">{date}</span>
      </nav>

      <h1 className="text-3xl font-bold mb-8 text-balance">{date}</h1>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 mb-6">
          {error}
        </div>
      )}

      {/* Daily Summary */}
      {summary && summary.overview && (
        <motion.section
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="mb-8"
        >
          <h2 className="text-xl font-semibold text-orange-400 mb-4">
            Daily Overview
          </h2>
          <div className="bg-daily-light rounded-lg p-6 border border-orange-500/20">
            <div className="markdown-content">
              <MarkdownRenderer content={summary.overview} />
            </div>

            {summary.insights && (
              <div className="mt-6 pt-6 border-t border-gray-800">
                <h3 className="text-lg font-medium text-gray-200 mb-3">
                  Key Insights
                </h3>
                <div className="markdown-content text-gray-400">
                  <MarkdownRenderer content={summary.insights} />
                </div>
              </div>
            )}

            {summary.tomorrow_focus && (
              <div className="mt-6 pt-6 border-t border-gray-800">
                <h3 className="text-lg font-medium text-gray-200 mb-3">
                  Tomorrow's Focus
                </h3>
                <div className="markdown-content text-gray-400">
                  <MarkdownRenderer content={summary.tomorrow_focus} />
                </div>
              </div>
            )}
          </div>
        </motion.section>
      )}

      {/* Sessions */}
      <section>
        <h2 className="text-xl font-semibold text-gray-200 mb-4">
          Sessions ({sessions.length})
        </h2>

        {sessions.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-gray-500">No sessions for this day.</p>
          </div>
        ) : (
          <div className="space-y-3">
            {sessions.map((session, i) => (
              <motion.div
                key={session.name}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.05 }}
              >
                <SessionCard session={session} date={date!} />
              </motion.div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
