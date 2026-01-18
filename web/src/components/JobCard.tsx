import { cn } from '../lib/utils'
import type { Job } from '../hooks/useApi'

interface JobCardProps {
  job: Job
  onKill?: (id: string) => void
}

export function JobCard({ job, onKill }: JobCardProps) {
  const isRunning = job.status_type === 'running'
  const isFailed = job.status_type === 'failed'

  return (
    <div
      className={cn(
        'p-4 rounded-lg border transition-colors',
        'bg-daily-light',
        isRunning && 'border-orange-500/40',
        isFailed && 'border-red-500/40',
        !isRunning && !isFailed && 'border-gray-700'
      )}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            {/* Status indicator */}
            <span
              className={cn(
                'size-2 rounded-full',
                isRunning && 'bg-orange-500 animate-pulse',
                isFailed && 'bg-red-500',
                !isRunning && !isFailed && 'bg-green-500'
              )}
            />
            <span className="font-medium text-gray-100 truncate">
              {job.task_name}
            </span>
          </div>

          <div className="text-sm text-gray-500 space-y-1">
            <div className="font-mono text-xs truncate" title={job.id}>
              {job.id}
            </div>
            <div className="flex items-center gap-4">
              <span>Started: {job.started_at}</span>
              <span className="tabular-nums">{job.elapsed}</span>
            </div>
            {isFailed && (
              <div className="text-red-400 mt-2">{job.status}</div>
            )}
          </div>
        </div>

        {/* Actions */}
        {isRunning && onKill && (
          <button
            onClick={() => onKill(job.id)}
            className={cn(
              'px-3 py-1 text-sm rounded transition-colors',
              'bg-red-500/20 text-red-400 hover:bg-red-500/30'
            )}
            aria-label={`Kill job ${job.task_name}`}
          >
            Kill
          </button>
        )}
      </div>
    </div>
  )
}
