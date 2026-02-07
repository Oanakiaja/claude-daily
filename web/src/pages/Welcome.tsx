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
import type { DateItem, DailySummary, Job, DailyUsageData } from '../hooks/useApi'
import { cn } from '../lib/utils'
import { formatCost } from '../components/UsageCharts'
import { useLanguage } from '../contexts/LanguageContext'

export function Welcome() {
  const [days, setDays] = useState<DateItem[]>([])
  const [summaries, setSummaries] = useState<Map<string, DailySummary>>(new Map())
  const [dailyCostMap, setDailyCostMap] = useState<Map<string, number>>(new Map())
  const [loading, setLoading] = useState(true)
  const [autoSummarizeJobs, setAutoSummarizeJobs] = useState<Job[]>([])
  const [currentMonth, setCurrentMonth] = useState(new Date())
  const [slideDirection, setSlideDirection] = useState<'left' | 'right'>('left')
  const { fetchDates, fetchDailySummary, fetchJobs, fetchInsights } = useApi()
  const navigate = useNavigate()
  const { t } = useLanguage()

  const WEEKDAYS = [
    t('weekdays.mon'), t('weekdays.tue'), t('weekdays.wed'), t('weekdays.thu'),
    t('weekdays.fri'), t('weekdays.sat'), t('weekdays.sun'),
  ]

  useEffect(() => {
    const loadData = async () => {
      try {
        const dates = await fetchDates()
        setDays(dates)

        // Load summaries for dates that have digest
        const results = await Promise.all(
          dates
            .filter(d => d.has_digest)
            .map(async (d) => {
              try {
                const summary = await fetchDailySummary(d.date)
                return [d.date, summary] as const
              } catch {
                return null
              }
            })
        )

        const map = new Map<string, DailySummary>()
        for (const r of results) {
          if (r) map.set(r[0], r[1])
        }
        setSummaries(map)

        // Load usage data for cost display
        try {
          const insights = await fetchInsights(365)
          if (insights.usage_summary?.daily_usage) {
            const costMap = new Map<string, number>()
            for (const du of insights.usage_summary.daily_usage) {
              costMap.set(du.date, du.total_cost_usd)
            }
            setDailyCostMap(costMap)
          }
        } catch {
          // Usage data is optional
        }
      } catch (err) {
        console.error('Failed to load data:', err)
      } finally {
        setLoading(false)
      }
    }

    loadData()
  }, [fetchDates, fetchDailySummary, fetchInsights])

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

  const monthKey = useMemo(() => format(currentMonth, 'yyyy-MM'), [currentMonth])

  // Compute monthly cost for the current month
  const monthlyCost = useMemo(() => {
    let total = 0
    for (const [date, cost] of dailyCostMap) {
      if (date.startsWith(monthKey)) {
        total += cost
      }
    }
    return total
  }, [dailyCostMap, monthKey])

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
      <div className="max-w-6xl mx-auto px-6 py-8">
        <h1 className="text-3xl font-bold mb-2">
          <span className="text-orange-500 dark:text-orange-400">{t('welcome.title')}</span> {t('welcome.titleSuffix')}
        </h1>
        <p className="text-gray-400 mb-8 h-5 w-48 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
        <div className="flex items-center justify-between mb-6">
          <div className="h-8 w-40 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
          <div className="h-8 w-20 bg-gray-200 dark:bg-daily-light rounded animate-pulse" />
        </div>
        <div className="grid grid-cols-7 gap-1.5">
          {WEEKDAYS.map(d => (
            <div key={d} className="text-center text-xs font-medium text-gray-400 dark:text-gray-500 py-2">
              {d}
            </div>
          ))}
          {[...Array(35)].map((_, i) => (
            <div key={i} className="h-24 bg-gray-200 dark:bg-daily-light rounded-lg animate-pulse" />
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
              {t('welcome.emptyTitle')} <span className="text-orange-500 dark:text-orange-400">{t('welcome.emptyTitleHighlight')}</span>
            </h1>
            <p className="text-gray-500 dark:text-gray-400 text-lg max-w-md mx-auto">
              {t('welcome.emptySubtitle')}
            </p>
            <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-800">
              <p className="text-gray-500 text-sm">
                {t('welcome.emptyHint')}
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
          <span className="text-orange-500 dark:text-orange-400">{t('welcome.title')}</span> {t('welcome.titleSuffix')}
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          {t('welcome.subtitle', {
            count: days.length,
            dayWord: days.length === 1 ? t('welcome.day') : t('welcome.days'),
          })}
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
                  {t('welcome.autoSummarizing', {
                    count: autoSummarizeJobs.length,
                    plural: autoSummarizeJobs.length > 1 ? 's' : '',
                  })}
                </p>
                <p className="text-blue-400/70 text-sm mt-1">
                  {t('welcome.autoSummarizingDesc')}
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
        <div className="flex items-center gap-3">
          {monthlyCost > 0 && (
            <span className="text-sm font-medium text-purple-500">
              {formatCost(monthlyCost)}
            </span>
          )}
          <button
            onClick={goToToday}
            className="px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-daily-light transition-colors text-gray-600 dark:text-gray-300"
          >
            {t('welcome.today')}
          </button>
        </div>
      </div>

      {/* Weekday headers */}
      <div className="grid grid-cols-7 gap-1.5 mb-1">
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
          className="grid grid-cols-7 gap-1.5"
        >
          {calendarDays.map(day => {
            const dateStr = format(day, 'yyyy-MM-dd')
            const archive = archiveMap.get(dateStr)
            const summary = summaries.get(dateStr)
            const dayCost = dailyCostMap.get(dateStr)
            const isCurrentMonth = isSameMonth(day, currentMonth)
            const today = isToday(day)
            const hasArchive = !!archive

            return (
              <button
                key={dateStr}
                onClick={() => hasArchive && navigate(`/day/${dateStr}`)}
                disabled={!hasArchive}
                className={cn(
                  'h-24 rounded-lg p-2 flex flex-col items-start text-left transition-all duration-150 relative overflow-hidden',
                  !isCurrentMonth && 'opacity-30',
                  hasArchive && 'cursor-pointer hover:ring-1 hover:ring-orange-500/50 hover:scale-[1.02] active:scale-[0.98]',
                  !hasArchive && 'cursor-default',
                  hasArchive
                    ? 'bg-orange-500/5 dark:bg-orange-500/10 border border-orange-500/20 dark:border-orange-500/15'
                    : 'border border-transparent',
                )}
              >
                {/* Header row: day number + session count */}
                <div className="flex items-center justify-between w-full mb-1">
                  <span
                    className={cn(
                      'text-xs font-semibold leading-none',
                      today && 'bg-orange-500 text-white rounded-full size-5 flex items-center justify-center text-[10px]',
                      !today && hasArchive && 'text-gray-900 dark:text-gray-100',
                      !today && !hasArchive && 'text-gray-400 dark:text-gray-600',
                    )}
                  >
                    {format(day, 'd')}
                  </span>

                  {hasArchive && (
                    <div className="flex items-center gap-1">
                      {archive.has_digest && (
                        <span className="size-1.5 rounded-full bg-orange-500 dark:bg-orange-400" />
                      )}
                      <span className="text-[10px] font-medium text-orange-500 dark:text-orange-400 leading-none">
                        {archive.session_count}s
                      </span>
                    </div>
                  )}
                </div>

                {/* Daily cost */}
                {dayCost != null && dayCost > 0 && (
                  <span className="text-[10px] font-medium text-purple-500 dark:text-purple-400 leading-none mb-0.5">
                    {formatCost(dayCost)}
                  </span>
                )}

                {/* Summary preview */}
                {hasArchive && summary?.overview && (
                  <p className="text-[10px] leading-tight text-gray-500 dark:text-gray-400 line-clamp-2 w-full">
                    {summary.overview}
                  </p>
                )}
              </button>
            )
          })}
        </motion.div>
      </AnimatePresence>
    </div>
  )
}
