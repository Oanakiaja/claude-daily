import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { SessionDetail as SessionDetailType } from '../hooks/useApi'
import { MarkdownRenderer } from '../components/MarkdownRenderer'

export function SessionDetail() {
  const { date, name } = useParams<{ date: string; name: string }>()
  const [session, setSession] = useState<SessionDetailType | null>(null)
  const { fetchSession, loading, error } = useApi()

  useEffect(() => {
    if (!date || !name) return
    fetchSession(date, name)
      .then(setSession)
      .catch(console.error)
  }, [date, name, fetchSession])

  if (loading && !session) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="animate-pulse space-y-4">
          <div className="h-8 w-64 bg-daily-light rounded" />
          <div className="h-64 bg-daily-light rounded-lg" />
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
        <Link to={`/day/${date}`} className="text-gray-500 hover:text-gray-300">
          {date}
        </Link>
        <span className="text-gray-600 mx-2">/</span>
        <span className="text-orange-400 truncate">{decodeURIComponent(name || '')}</span>
      </nav>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 mb-6">
          {error}
        </div>
      )}

      {session && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          {/* Session Metadata */}
          <div className="mb-8">
            <h1 className="text-2xl font-bold mb-4 text-balance">
              {session.metadata?.title || decodeURIComponent(name || '')}
            </h1>

            <div className="flex flex-wrap gap-4 text-sm text-gray-400">
              {session.metadata?.cwd && (
                <div className="flex items-center gap-2">
                  <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  <span className="font-mono text-xs truncate max-w-xs">
                    {session.metadata.cwd}
                  </span>
                </div>
              )}
              {session.metadata?.git_branch && (
                <div className="flex items-center gap-2">
                  <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h10v10H7z" />
                  </svg>
                  <span>{session.metadata.git_branch}</span>
                </div>
              )}
              {session.metadata?.duration && (
                <div className="flex items-center gap-2">
                  <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span>{session.metadata.duration}</span>
                </div>
              )}
              {session.metadata?.tool_calls !== undefined && (
                <div className="flex items-center gap-2">
                  <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  <span>{session.metadata.tool_calls} tool calls</span>
                </div>
              )}
            </div>
          </div>

          {/* Session Content */}
          <div className="bg-daily-light rounded-lg p-6 border border-orange-500/20">
            <div className="markdown-content">
              <MarkdownRenderer content={session.content} />
            </div>
          </div>
        </motion.div>
      )}
    </div>
  )
}
