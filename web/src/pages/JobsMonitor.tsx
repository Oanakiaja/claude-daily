import { useState, useEffect, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { Job } from '../hooks/useApi'
import { JobCard } from '../components/JobCard'
import { useLanguage } from '../contexts/LanguageContext'

export function JobsMonitor() {
  const [jobs, setJobs] = useState<Job[]>([])
  const { fetchJobs, killJob, error } = useApi()
  const { t } = useLanguage()

  const loadJobs = useCallback(() => {
    fetchJobs()
      .then(setJobs)
      .catch(console.error)
  }, [fetchJobs])

  useEffect(() => {
    loadJobs()
    // Poll every 5 seconds as fallback
    const interval = setInterval(loadJobs, 5000)
    return () => clearInterval(interval)
  }, [loadJobs])

  const handleKill = async (id: string) => {
    try {
      await killJob(id)
      loadJobs()
    } catch (err) {
      console.error('Failed to kill job:', err)
    }
  }

  const runningJobs = jobs.filter((j) => j.status_type === 'running')
  const completedJobs = jobs.filter((j) => j.status_type !== 'running')

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-balance">{t('jobs.title')}</h1>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 mb-6">
          {error}
        </div>
      )}

      {/* Running Jobs */}
      <section className="mb-8">
        <h2 className="text-xl font-semibold text-orange-500 dark:text-orange-400 mb-4">
          {t('jobs.running')} ({runningJobs.length})
        </h2>
        <AnimatePresence mode="popLayout">
          {runningJobs.length === 0 ? (
            <motion.p
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-gray-500"
            >
              {t('jobs.noRunning')}
            </motion.p>
          ) : (
            <div className="space-y-3">
              {runningJobs.map((job) => (
                <motion.div
                  key={job.id}
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  layout
                >
                  <JobCard job={job} onKill={handleKill} />
                </motion.div>
              ))}
            </div>
          )}
        </AnimatePresence>
      </section>

      {/* Completed Jobs */}
      <section>
        <h2 className="text-xl font-semibold text-gray-400 mb-4">
          {t('jobs.completed')} ({completedJobs.length})
        </h2>
        {completedJobs.length === 0 ? (
          <p className="text-gray-500">{t('jobs.noCompleted')}</p>
        ) : (
          <div className="space-y-3">
            {completedJobs.map((job, i) => (
              <motion.div
                key={job.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.03 }}
              >
                <JobCard job={job} />
              </motion.div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
