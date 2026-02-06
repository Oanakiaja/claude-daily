import { useState } from 'react'
import { Link } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import type { SessionInsight } from '../hooks/useApi'

type FilterTab = 'all' | 'friction' | 'not_achieved' | 'low_satisfaction'

const FILTER_TABS: { key: FilterTab; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'friction', label: 'With Friction' },
  { key: 'not_achieved', label: 'Not Achieved' },
  { key: 'low_satisfaction', label: 'Low Satisfaction' },
]

function OutcomeBadge({ outcome }: { outcome: string | null }) {
  if (!outcome) return null

  const config: Record<string, { bg: string; text: string; label: string }> = {
    achieved: { bg: 'bg-green-100 dark:bg-green-900/30', text: 'text-green-600 dark:text-green-400', label: 'Achieved' },
    partially_achieved: { bg: 'bg-yellow-100 dark:bg-yellow-900/30', text: 'text-yellow-600 dark:text-yellow-400', label: 'Partial' },
    not_achieved: { bg: 'bg-red-100 dark:bg-red-900/30', text: 'text-red-600 dark:text-red-400', label: 'Not Achieved' },
  }

  const style = config[outcome] || { bg: 'bg-gray-100 dark:bg-gray-800', text: 'text-gray-500', label: outcome }

  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${style.bg} ${style.text}`}>
      {style.label}
    </span>
  )
}

function FrictionTag({ type: frictionType }: { type: string }) {
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400">
      {frictionType.replace(/_/g, ' ')}
    </span>
  )
}

function SatisfactionIndicator({ satisfaction }: { satisfaction: string | null }) {
  if (!satisfaction) return null

  const config: Record<string, { color: string; label: string }> = {
    happy: { color: 'text-green-500', label: 'Happy' },
    likely_satisfied: { color: 'text-green-400', label: 'Satisfied' },
    satisfied: { color: 'text-green-400', label: 'Satisfied' },
    neutral: { color: 'text-yellow-500', label: 'Neutral' },
    frustrated: { color: 'text-red-500', label: 'Frustrated' },
  }

  const style = config[satisfaction] || { color: 'text-gray-400', label: satisfaction }

  return (
    <span className={`text-xs font-medium ${style.color}`}>
      {style.label}
    </span>
  )
}

/** Compute a severity score for sorting: higher = more problematic */
function severityScore(s: SessionInsight): number {
  let score = 0
  if (s.friction_types.length > 0) score += 100 + s.friction_types.length * 10
  if (s.outcome === 'not_achieved') score += 80
  if (s.outcome === 'partially_achieved') score += 30
  if (s.satisfaction === 'frustrated') score += 60
  if (s.satisfaction === 'neutral') score += 20
  if (s.claude_helpfulness === 'not_helpful') score += 40
  if (s.claude_helpfulness === 'slightly_helpful') score += 20
  return score
}

function filterSessions(sessions: SessionInsight[], filter: FilterTab): SessionInsight[] {
  let filtered: SessionInsight[]

  switch (filter) {
    case 'friction':
      filtered = sessions.filter(s => s.friction_types.length > 0)
      break
    case 'not_achieved':
      filtered = sessions.filter(s => s.outcome === 'not_achieved')
      break
    case 'low_satisfaction':
      filtered = sessions.filter(
        s => s.satisfaction === 'frustrated' || s.satisfaction === 'neutral'
      )
      break
    default:
      filtered = sessions
  }

  // Sort by severity (most problematic first)
  return [...filtered].sort((a, b) => severityScore(b) - severityScore(a))
}

function SessionCard({ session }: { session: SessionInsight }) {
  const hasFriction = session.friction_types.length > 0
  const hasProblems = hasFriction || session.outcome === 'not_achieved' || session.satisfaction === 'frustrated'

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className={`bg-gray-50 dark:bg-daily-light rounded-xl p-5 border ${
        hasProblems
          ? 'border-red-200 dark:border-red-900/40'
          : 'border-gray-200 dark:border-gray-800'
      }`}
    >
      {/* Header: session name + date */}
      <div className="flex items-start justify-between gap-3 mb-3">
        <Link
          to={`/day/${session.date}/session/${encodeURIComponent(session.session_name)}`}
          className="text-sm font-medium text-orange-500 dark:text-orange-400 hover:underline break-all leading-tight"
        >
          {session.session_name}
        </Link>
        <span className="shrink-0 text-xs px-2 py-0.5 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400">
          {session.date}
        </span>
      </div>

      {/* Brief summary */}
      {session.brief_summary && (
        <p className="text-sm text-gray-600 dark:text-gray-300 mb-3 leading-relaxed">
          {session.brief_summary}
        </p>
      )}

      {/* Badges row: outcome, satisfaction, session type */}
      <div className="flex flex-wrap items-center gap-2 mb-2">
        <OutcomeBadge outcome={session.outcome} />
        <SatisfactionIndicator satisfaction={session.satisfaction} />
        {session.claude_helpfulness && (
          <span className="text-xs text-gray-400 dark:text-gray-500">
            Claude: {session.claude_helpfulness.replace(/_/g, ' ')}
          </span>
        )}
        {session.session_type && (
          <span className="text-xs text-gray-400 dark:text-gray-500">
            {session.session_type.replace(/_/g, ' ')}
          </span>
        )}
      </div>

      {/* Friction tags */}
      {hasFriction && (
        <div className="flex flex-wrap gap-1.5 mt-2">
          {session.friction_types.map(ft => (
            <FrictionTag key={ft} type={ft} />
          ))}
        </div>
      )}

      {/* Friction detail */}
      {session.friction_detail && (
        <p className="text-xs text-red-500 dark:text-red-400 mt-2 leading-relaxed italic">
          {session.friction_detail}
        </p>
      )}

      {/* Goal categories */}
      {session.goal_categories.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mt-2">
          {session.goal_categories.map(gc => (
            <span
              key={gc}
              className="text-xs px-2 py-0.5 rounded-full bg-orange-100 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400"
            >
              {gc.replace(/_/g, ' ')}
            </span>
          ))}
        </div>
      )}
    </motion.div>
  )
}

interface InsightsProblemViewProps {
  sessionDetails: SessionInsight[]
}

export function InsightsProblemView({ sessionDetails }: InsightsProblemViewProps) {
  const [activeFilter, setActiveFilter] = useState<FilterTab>('all')

  const filtered = filterSessions(sessionDetails, activeFilter)

  // Count badges for filter tabs
  const frictionCount = sessionDetails.filter(s => s.friction_types.length > 0).length
  const notAchievedCount = sessionDetails.filter(s => s.outcome === 'not_achieved').length
  const lowSatisfactionCount = sessionDetails.filter(
    s => s.satisfaction === 'frustrated' || s.satisfaction === 'neutral'
  ).length

  const counts: Record<FilterTab, number | null> = {
    all: null,
    friction: frictionCount,
    not_achieved: notAchievedCount,
    low_satisfaction: lowSatisfactionCount,
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="mt-8"
    >
      {/* Section header */}
      <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">
        Session Details
      </h2>

      {/* Filter tabs - only show tabs that have matching sessions */}
      <div className="flex flex-wrap gap-2 mb-5">
        {FILTER_TABS.filter(tab => tab.key === 'all' || (counts[tab.key] ?? 0) > 0).map(tab => {
          const count = counts[tab.key]
          const isActive = activeFilter === tab.key
          return (
            <button
              key={tab.key}
              onClick={() => setActiveFilter(tab.key)}
              className={`px-3 py-1.5 text-sm rounded-lg transition-colors flex items-center gap-1.5 ${
                isActive
                  ? 'bg-orange-500 text-white'
                  : 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'
              }`}
            >
              {tab.label}
              {count !== null && count > 0 && (
                <span
                  className={`text-xs px-1.5 py-0.5 rounded-full ${
                    isActive
                      ? 'bg-white/20 text-white'
                      : 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400'
                  }`}
                >
                  {count}
                </span>
              )}
            </button>
          )
        })}
      </div>

      {/* Session cards grid */}
      <AnimatePresence mode="popLayout">
        {filtered.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {filtered.map(session => (
              <SessionCard key={`${session.date}-${session.session_name}`} session={session} />
            ))}
          </div>
        ) : (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-center py-12 bg-gray-50 dark:bg-daily-light rounded-xl border border-gray-200 dark:border-gray-800"
          >
            <p className="text-gray-500 dark:text-gray-400">
              No issues detected in this period
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  )
}
