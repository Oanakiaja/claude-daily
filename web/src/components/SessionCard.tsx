import { Link } from 'react-router-dom'
import { cn } from '../lib/utils'
import type { Session } from '../hooks/useApi'

interface SessionCardProps {
  session: Session
  date: string
}

export function SessionCard({ session, date }: SessionCardProps) {
  return (
    <Link to={`/day/${date}/session/${encodeURIComponent(session.name)}`}>
      <div
        className={cn(
          'p-4 rounded-lg border transition-colors',
          'bg-daily-light border-orange-500/20 hover:border-orange-500/40'
        )}
      >
        <h3 className="font-medium text-gray-100 mb-2 line-clamp-1">
          {session.title || session.name}
        </h3>
        {session.summary_preview && (
          <p className="text-sm text-gray-400 line-clamp-2 text-pretty">
            {session.summary_preview}
          </p>
        )}
      </div>
    </Link>
  )
}
