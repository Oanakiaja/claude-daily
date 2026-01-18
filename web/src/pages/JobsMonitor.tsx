import { useState, useEffect, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { Job } from '../hooks/useApi'
import { useWebSocket } from '../hooks/useWebSocket'
import { JobCard } from '../components/JobCard'

interface WebSocketJobMessage {
  type: string
  data: Job
}

export function JobsMonitor() {
  const [jobs, setJobs] = useState<Job[]>([])
  const { fetchJobs, killJob, error } = useApi()

  const handleWsMessage = useCallback((msg: WebSocketJobMessage) => {
    if (msg.type === 'JobUpdated') {
      setJobs((prev) => {
        const idx = prev.findIndex((j) => j.id === msg.data.id)
        if (idx >= 0) {
          const updated = [...prev]
          updated[idx] = msg.data
          return updated
        }
        return [msg.data, ...prev]
      })
    }
  }, [])

  const { connected } = useWebSocket(handleWsMessage as (msg: { type: string; data?: unknown }) => void)

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
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-3xl font-bold text-balance">Jobs Monitor</h1>
        <div className="flex items-center gap-2">
          <span
            className={`size-2 rounded-full ${
              connected ? 'bg-green-500' : 'bg-red-500'
            }`}
          />
          <span className="text-sm text-gray-500">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 mb-6">
          {error}
        </div>
      )}

      {/* Running Jobs */}
      <section className="mb-8">
        <h2 className="text-xl font-semibold text-orange-400 mb-4">
          Running ({runningJobs.length})
        </h2>
        <AnimatePresence mode="popLayout">
          {runningJobs.length === 0 ? (
            <motion.p
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-gray-500"
            >
              No running jobs
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
          Completed ({completedJobs.length})
        </h2>
        {completedJobs.length === 0 ? (
          <p className="text-gray-500">No completed jobs</p>
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
