import { useState, useEffect, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import {
  format,
  isToday,
  isSameMonth,
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  addMonths,
  subMonths,
} from 'date-fns'
import { useApi } from '../hooks/useApi'
import type { DateItem, Job } from '../hooks/useApi'
import { cn } from '../lib/utils'

const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

export function Welcome() {
  const [days, setDays] = useState<DateItem[]>([])
  const [loading, setLoading] = useState(true)
  const [autoSummarizeJobs, setAutoSummarizeJobs] = useState<Job[]>([])
  const [currentMonth, setCurrentMonth] = useState(new Date())
  const [slideDirection, setSlideDirection] = useState<'left' | 'right'>('left')
  const { fetchDates, fetchJobs } = useApi()
  const navigate = useNavigate()

  useEffect(() => {
    const loadData = async () => {
      try {
        const dates = await fetchDates()
        setDays(dates)
      } catch (err) {
        console.error('Failed to load data:', err)
      } finally {
        setLoading(false)
      }
    }

    loadData()
  }, [fetchDates])

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

  const archiveMap = useMemo(() => {
    const map = new Map<string, DateItem>()
    for (const day of days) {
      map.set(day.date, day)
    }
    return map
  }, [days])

  const calendarDays = useMemo(() => {
    const monthStart = startOfMonth(currentMonth)
    const monthEnd = endOfMonth(currentMonth)
    const calStart = startOfWeek(monthStart, { weekStartsOn: 1 })
    const calEnd = endOfWeek(monthEnd, { weekStartsOn: 1 })
    return eachDayOfInterval({ start: calStart, end: calEnd })
  }, [currentMonth])

  const goToPrevMonth = () => {
    setSlideDirection('right')
    setCurrentMonth(prev => subMonths(prev, 1))
  }

  const goToNextMonth = () => {
    setSlideDirection('left')
    setCurrentMonth(prev => addMonths(prev, 1))
  }

  const goToToday = () => {
    const today = new Date()
    if (currentMonth > today) {
      setSlideDirection('right')
    } else {
      setSlideDirection('left')
    }
    setCurrentMonth(today)
  }

  const slideVariants = {
    enter: (direction: 'left' | 'right') => ({
      x: direction === 'left' ? 80 : -80,
      opacity: 0,
    }),
    center: {
      x: 0,
      opacity: 1,
    },
    exit: (direction: 'left' | 'right') => ({
      x: direction === 'left' ? -80 : 80,
      opacity: 0,
    }),
  }

  if (loading) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <h1 className="text-3xl font-bold mb-2">
          <span className="text-orange-500 dark:text-orange-400">Daily</span> Archives
        </h1>
        <p className="text-gray-400 mb-8 h-5 w-48 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
        <div className="flex items-center justify-between mb-6">
          <div className="h-8 w-40 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
          <div className="h-8 w-20 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
        </div>
        <div className="grid grid-cols-7 gap-1">
          {WEEKDAYS.map(d => (
            <div key={d} className="text-center text-xs font-medium text-gray-400 dark:text-gray-500 py-2">
              {d}
            </div>
          ))}
          {[...Array(35)].map((_, i) => (
            <div key={i} className="aspect-square bg-gray-200 dark:bg-daily-light rounded-lg animate-pulse" />
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

  const monthKey = format(currentMonth, 'yyyy-MM')

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
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

      {/* Month navigation */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <button
            onClick={goToPrevMonth}
            className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-daily-light transition-colors"
            aria-label="Previous month"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" className="text-gray-500 dark:text-gray-400">
              <path d="M12.5 15L7.5 10L12.5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
          <h2 className="text-lg font-semibold min-w-[180px] text-center">
            {format(currentMonth, 'MMMM yyyy')}
          </h2>
          <button
            onClick={goToNextMonth}
            className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-daily-light transition-colors"
            aria-label="Next month"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" className="text-gray-500 dark:text-gray-400">
              <path d="M7.5 15L12.5 10L7.5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </div>
        <button
          onClick={goToToday}
          className="px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-daily-light transition-colors text-gray-600 dark:text-gray-300"
        >
          Today
        </button>
      </div>

      {/* Weekday headers */}
      <div className="grid grid-cols-7 gap-1 mb-1">
        {WEEKDAYS.map(d => (
          <div key={d} className="text-center text-xs font-medium text-gray-400 dark:text-gray-500 py-2">
            {d}
          </div>
        ))}
      </div>

      {/* Calendar grid */}
      <AnimatePresence mode="wait" custom={slideDirection}>
        <motion.div
          key={monthKey}
          custom={slideDirection}
          variants={slideVariants}
          initial="enter"
          animate="center"
          exit="exit"
          transition={{ duration: 0.2, ease: 'easeInOut' }}
          className="grid grid-cols-7 gap-1"
        >
          {calendarDays.map(day => {
            const dateStr = format(day, 'yyyy-MM-dd')
            const archive = archiveMap.get(dateStr)
            const isCurrentMonth = isSameMonth(day, currentMonth)
            const today = isToday(day)
            const hasArchive = !!archive

            return (
              <button
                key={dateStr}
                onClick={() => hasArchive && navigate(`/day/${dateStr}`)}
                disabled={!hasArchive}
                className={cn(
                  'aspect-square rounded-lg p-1.5 flex flex-col items-center justify-center gap-0.5 transition-all duration-150 relative',
                  !isCurrentMonth && 'opacity-30',
                  hasArchive && 'cursor-pointer hover:bg-orange-500/10 hover:scale-105 active:scale-95',
                  !hasArchive && 'cursor-default',
                  hasArchive && 'bg-orange-500/5 dark:bg-orange-500/10',
                )}
              >
                {/* Day number */}
                <span
                  className={cn(
                    'text-sm font-medium leading-none',
                    today && 'bg-orange-500 text-white rounded-full size-6 flex items-center justify-center',
                    !today && hasArchive && 'text-gray-900 dark:text-gray-100',
                    !today && !hasArchive && 'text-gray-400 dark:text-gray-600',
                  )}
                >
                  {format(day, 'd')}
                </span>

                {/* Session count */}
                {hasArchive && (
                  <span className="text-[10px] font-medium text-orange-500 dark:text-orange-400 leading-none">
                    {archive.session_count}s
                  </span>
                )}

                {/* Digest dot */}
                {archive?.has_digest && (
                  <span className="size-1 rounded-full bg-orange-500 dark:bg-orange-400" />
                )}
              </button>
            )
          })}
        </motion.div>
      </AnimatePresence>
    </div>
  )
}
