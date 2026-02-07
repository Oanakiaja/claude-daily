import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { DateItem } from '../hooks/useApi'
import { cn } from '../lib/utils'
import { format, parseISO, isToday, isYesterday } from 'date-fns'
import { useLanguage } from '../contexts/LanguageContext'

export function DailyList() {
  const [dates, setDates] = useState<DateItem[]>([])
  const { fetchDates, loading, error } = useApi()
  const { t } = useLanguage()

  useEffect(() => {
    fetchDates()
      .then(setDates)
      .catch(console.error)
  }, [fetchDates])

  const getDateLabel = (dateStr: string) => {
    const date = parseISO(dateStr)
    if (isToday(date)) return t('archive.today')
    if (isYesterday(date)) return t('archive.yesterday')
    return format(date, 'EEEE')
  }

  if (loading && dates.length === 0) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="animate-pulse space-y-4">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="h-20 bg-gray-200 dark:bg-daily-light rounded-lg" />
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-8">
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400">
          {t('archive.loadFailed')} {error}
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      <h1 className="text-3xl font-bold mb-8 text-balance">{t('archive.title')}</h1>

      {dates.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-gray-500 text-lg">{t('archive.noArchives')}</p>
          <p className="text-gray-600 text-sm mt-2">
            {t('archive.noArchivesHint')}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {dates.map((item, i) => (
            <motion.div
              key={item.date}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.03, duration: 0.2 }}
            >
              <Link
                to={`/day/${item.date}`}
                className={cn(
                  'block p-4 rounded-lg border transition-colors',
                  'bg-gray-50 dark:bg-daily-light border-gray-200 dark:border-orange-500/20 hover:border-orange-400 dark:hover:border-orange-500/40'
                )}
              >
                <div className="flex justify-between items-start">
                  <div>
                    <div className="flex items-center gap-3">
                      <span className="text-lg font-medium tabular-nums">
                        {item.date}
                      </span>
                      <span className="text-sm text-gray-500">
                        {getDateLabel(item.date)}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 mt-1">
                      <span className="text-sm text-gray-400">
                        {item.session_count} {item.session_count === 1 ? t('archive.session') : t('archive.sessions')}
                      </span>
                      {item.has_digest && (
                        <span className="text-xs text-orange-400 bg-orange-500/10 px-2 py-0.5 rounded">
                          {t('archive.digest')}
                        </span>
                      )}
                    </div>
                  </div>
                  <svg
                    className="size-5 text-gray-600"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 5l7 7-7 7"
                    />
                  </svg>
                </div>
              </Link>
            </motion.div>
          ))}
        </div>
      )}
    </div>
  )
}
