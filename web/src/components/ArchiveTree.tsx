import { useState, useEffect } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { format, parseISO, isToday, isYesterday } from 'date-fns'
import { useApi } from '../hooks/useApi'
import type { DateItem, Session } from '../hooks/useApi'
import { cn } from '../lib/utils'

interface DateNodeState {
  expanded: boolean
  sessions: Session[]
  sessionsLoaded: boolean
}

export function ArchiveTree() {
  const [dates, setDates] = useState<DateItem[]>([])
  const [dateStates, setDateStates] = useState<Record<string, DateNodeState>>({})
  const { fetchDates, fetchSessions, loading } = useApi()
  const navigate = useNavigate()
  const location = useLocation()

  // Load dates on mount
  useEffect(() => {
    fetchDates()
      .then((data) => {
        setDates(data)
        // Auto-expand today's date
        const today = data.find(d => isToday(parseISO(d.date)))
        if (today) {
          setDateStates(prev => ({
            ...prev,
            [today.date]: { expanded: true, sessions: [], sessionsLoaded: false }
          }))
        }
      })
      .catch(console.error)
  }, [fetchDates])

  // Load sessions when a date is expanded
  useEffect(() => {
    const expandedDates = Object.entries(dateStates)
      .filter(([_, state]) => state.expanded && !state.sessionsLoaded)
      .map(([date]) => date)

    expandedDates.forEach(date => {
      fetchSessions(date)
        .then(sessions => {
          setDateStates(prev => ({
            ...prev,
            [date]: { ...prev[date], sessions, sessionsLoaded: true }
          }))
        })
        .catch(console.error)
    })
  }, [dateStates, fetchSessions])

  const toggleDate = (date: string) => {
    setDateStates(prev => ({
      ...prev,
      [date]: {
        expanded: !prev[date]?.expanded,
        sessions: prev[date]?.sessions || [],
        sessionsLoaded: prev[date]?.sessionsLoaded || false
      }
    }))
  }

  const getDateLabel = (dateStr: string) => {
    const date = parseISO(dateStr)
    if (isToday(date)) return 'Today'
    if (isYesterday(date)) return 'Yesterday'
    return format(date, 'EEEE')
  }

  const isActive = (path: string) => location.pathname === path

  if (loading && dates.length === 0) {
    return (
      <div className="p-4 space-y-2">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="h-12 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
        ))}
      </div>
    )
  }

  return (
    <div className="h-full overflow-y-auto p-4 space-y-1">
      {dates.map((dateItem) => {
        const state = dateStates[dateItem.date] || { expanded: false, sessions: [], sessionsLoaded: false }

        return (
          <div key={dateItem.date}>
            {/* Date header */}
            <button
              onClick={() => toggleDate(dateItem.date)}
              className={cn(
                'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors',
                'hover:bg-gray-100 dark:hover:bg-daily-light'
              )}
            >
              <svg
                className={cn(
                  'size-4 transition-transform shrink-0',
                  state.expanded && 'rotate-90'
                )}
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>

              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-sm tabular-nums">{dateItem.date}</span>
                  <span className="text-xs text-gray-500">{getDateLabel(dateItem.date)}</span>
                </div>
                <div className="text-xs text-gray-500 mt-0.5">
                  {dateItem.session_count} {dateItem.session_count === 1 ? 'session' : 'sessions'}
                </div>
              </div>
            </button>

            {/* Expanded content */}
            <AnimatePresence>
              {state.expanded && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  <div className="ml-6 mt-1 space-y-0.5">
                    {/* Daily Summary */}
                    <button
                      onClick={() => navigate(`/day/${dateItem.date}`)}
                      className={cn(
                        'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors',
                        isActive(`/day/${dateItem.date}`)
                          ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400 border border-orange-500/30'
                          : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-700 dark:text-gray-300'
                      )}
                    >
                      <span className="text-base">📝</span>
                      <span>Daily Summary</span>
                    </button>

                    {/* Sessions */}
                    {state.sessionsLoaded ? (
                      state.sessions.length > 0 && (
                        <div className="pt-1">
                          <div className="px-3 py-1 text-xs text-gray-500 font-medium">
                            Sessions ({state.sessions.length})
                          </div>
                          {state.sessions.map((session) => (
                            <button
                              key={session.name}
                              onClick={() => navigate(`/day/${dateItem.date}/session/${encodeURIComponent(session.name)}`)}
                              className={cn(
                                'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors',
                                isActive(`/day/${dateItem.date}/session/${encodeURIComponent(session.name)}`)
                                  ? 'bg-orange-500/20 text-orange-500 dark:text-orange-400 border border-orange-500/30'
                                  : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-500 dark:text-gray-400'
                              )}
                              title={session.title || session.name}
                            >
                              <span className="text-base">📄</span>
                              <span className="truncate">{session.title || session.name}</span>
                            </button>
                          ))}
                        </div>
                      )
                    ) : (
                      <div className="px-3 py-2 text-xs text-gray-500">Loading sessions...</div>
                    )}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )
      })}
    </div>
  )
}
