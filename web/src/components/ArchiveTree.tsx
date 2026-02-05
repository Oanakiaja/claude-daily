import { useState, useEffect, useMemo, useCallback } from 'react'
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

type NavItem =
  | { type: 'date'; date: string }
  | { type: 'daily'; date: string; path: string }
  | { type: 'session'; date: string; name: string; path: string }

export function ArchiveTree() {
  const [dates, setDates] = useState<DateItem[]>([])
  const [dateStates, setDateStates] = useState<Record<string, DateNodeState>>({})
  const [focusedIndex, setFocusedIndex] = useState<number>(-1)
  const { fetchDates, fetchSessions, loading } = useApi()
  const navigate = useNavigate()
  const location = useLocation()

  // Build flat navigation list from tree structure
  const navItems = useMemo<NavItem[]>(() => {
    const items: NavItem[] = []
    dates.forEach(dateItem => {
      items.push({ type: 'date', date: dateItem.date })
      const state = dateStates[dateItem.date]
      if (state?.expanded) {
        items.push({ type: 'daily', date: dateItem.date, path: `/day/${dateItem.date}` })
        if (state.sessionsLoaded && state.sessions.length > 0) {
          state.sessions.forEach(session => {
            items.push({
              type: 'session',
              date: dateItem.date,
              name: session.name,
              path: `/day/${dateItem.date}/session/${encodeURIComponent(session.name)}`
            })
          })
        }
      }
    })
    return items
  }, [dates, dateStates])

  // Find current focused index based on location
  const findCurrentIndex = useCallback(() => {
    const path = location.pathname
    return navItems.findIndex(item => {
      if (item.type === 'daily' || item.type === 'session') {
        return item.path === path
      }
      return false
    })
  }, [navItems, location.pathname])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Only handle when on Archives page
      if (!location.pathname.startsWith('/day') && location.pathname !== '/') return

      // Skip if user is typing
      const target = e.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return
      }

      if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
        e.preventDefault()

        setFocusedIndex(prev => {
          const currentIdx = prev === -1 ? findCurrentIndex() : prev
          if (e.key === 'ArrowUp') {
            return currentIdx > 0 ? currentIdx - 1 : navItems.length - 1
          } else {
            return currentIdx < navItems.length - 1 ? currentIdx + 1 : 0
          }
        })
      } else if (e.key === 'Enter' && focusedIndex >= 0) {
        e.preventDefault()
        const item = navItems[focusedIndex]
        if (item.type === 'date') {
          toggleDate(item.date)
        } else {
          navigate(item.path)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [navItems, focusedIndex, findCurrentIndex, location.pathname, navigate])

  // Reset focus when navigating
  useEffect(() => {
    setFocusedIndex(-1)
  }, [location.pathname])

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

  // Get focused item index for a specific item
  const getItemIndex = (type: NavItem['type'], date: string, sessionName?: string) => {
    return navItems.findIndex(item => {
      if (item.type !== type) return false
      if (item.date !== date) return false
      if (type === 'session' && item.type === 'session') {
        return item.name === sessionName
      }
      return true
    })
  }

  return (
    <div className="h-full overflow-y-auto p-4 space-y-1">
      {dates.map((dateItem) => {
        const state = dateStates[dateItem.date] || { expanded: false, sessions: [], sessionsLoaded: false }
        const dateIndex = getItemIndex('date', dateItem.date)

        return (
          <div key={dateItem.date}>
            {/* Date header */}
            <button
              onClick={() => toggleDate(dateItem.date)}
              className={cn(
                'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors',
                'hover:bg-gray-100 dark:hover:bg-daily-light',
                focusedIndex === dateIndex && 'ring-2 ring-orange-500 bg-orange-500/10'
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
                          : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-700 dark:text-gray-300',
                        focusedIndex === getItemIndex('daily', dateItem.date) && 'ring-2 ring-orange-500'
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
                                  : 'hover:bg-gray-100 dark:hover:bg-daily-light text-gray-500 dark:text-gray-400',
                                focusedIndex === getItemIndex('session', dateItem.date, session.name) && 'ring-2 ring-orange-500'
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
