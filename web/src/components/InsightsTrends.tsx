import { motion } from 'framer-motion'
import type { TrendData } from '../hooks/useApi'
import { useLanguage } from '../contexts/LanguageContext'

interface InsightsTrendsProps {
  trends: TrendData | undefined
}

/** Format a percentage change value with sign */
function formatChange(pct: number): string {
  if (Math.abs(pct) < 0.1) return '0%'
  const sign = pct > 0 ? '+' : ''
  return `${sign}${pct.toFixed(1)}%`
}

/** Get arrow direction character */
function getArrow(pct: number): string {
  if (Math.abs(pct) < 0.1) return '\u2192' // right arrow for neutral
  return pct > 0 ? '\u2191' : '\u2193' // up or down
}

/**
 * Get color classes for a change indicator.
 * For most metrics, positive change is good (green) and negative is bad (red).
 * For friction, the polarity is inverted: friction going DOWN is good.
 */
function getChangeColor(pct: number, invertPolarity: boolean = false): string {
  if (Math.abs(pct) < 0.1) return 'text-gray-400'
  const isPositive = pct > 0
  const isGood = invertPolarity ? !isPositive : isPositive
  return isGood ? 'text-green-500' : 'text-red-500'
}

/** Get background accent color for the change badge */
function getChangeBg(pct: number, invertPolarity: boolean = false): string {
  if (Math.abs(pct) < 0.1) return 'bg-gray-100 dark:bg-gray-800'
  const isPositive = pct > 0
  const isGood = invertPolarity ? !isPositive : isPositive
  return isGood ? 'bg-green-50 dark:bg-green-900/20' : 'bg-red-50 dark:bg-red-900/20'
}

interface TrendCardProps {
  label: string
  value: string
  changePct: number
  comparisonLabel: string
  /** When true, a decrease is considered good (e.g. friction rate) */
  invertPolarity?: boolean
}

function TrendCard({ label, value, changePct, comparisonLabel, invertPolarity = false }: TrendCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-gray-50 dark:bg-daily-light rounded-xl p-5 border border-gray-200 dark:border-gray-800"
    >
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">{label}</p>
      <p className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">{value}</p>
      <div className="flex items-center gap-2">
        <span
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${getChangeBg(changePct, invertPolarity)} ${getChangeColor(changePct, invertPolarity)}`}
        >
          <span>{getArrow(changePct)}</span>
          <span>{formatChange(changePct)}</span>
        </span>
        <span className="text-xs text-gray-400 dark:text-gray-500">{comparisonLabel}</span>
      </div>
    </motion.div>
  )
}

export function InsightsTrends({ trends }: InsightsTrendsProps) {
  const { t } = useLanguage()

  if (!trends) {
    return (
      <div className="mb-6 p-4 bg-gray-50 dark:bg-daily-light rounded-xl border border-gray-200 dark:border-gray-800 text-center">
        <p className="text-sm text-gray-400 dark:text-gray-500">
          {t('insights.trendNoData')}
        </p>
      </div>
    )
  }

  const compLabel = trends.comparison_label

  return (
    <div className="mb-6">
      <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-3">
        {t('insights.trendAnalysis')}
        <span className="ml-2 text-xs text-gray-400 dark:text-gray-500">
          ({trends.period_label})
        </span>
      </h3>

      {/* Trend Summary Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
        <TrendCard
          label={t('insights.trendSessions')}
          value={String(trends.current_sessions)}
          changePct={trends.sessions_change_pct}
          comparisonLabel={compLabel}
        />
        <TrendCard
          label={t('insights.trendFrictionRate')}
          value={`${trends.current_friction_rate.toFixed(1)}%`}
          changePct={trends.friction_change_pct}
          comparisonLabel={compLabel}
          invertPolarity
        />
        <TrendCard
          label={t('insights.trendSuccessRate')}
          value={`${trends.current_success_rate.toFixed(1)}%`}
          changePct={trends.success_change_pct}
          comparisonLabel={compLabel}
        />
        <TrendCard
          label={t('insights.trendSatisfaction')}
          value={trends.current_satisfaction_score > 0 ? trends.current_satisfaction_score.toFixed(0) : 'N/A'}
          changePct={trends.current_satisfaction_score > 0 ? trends.satisfaction_change_pct : 0}
          comparisonLabel={compLabel}
        />
      </div>

      {/* Weekly Breakdown Table */}
      {trends.weekly_stats.length > 1 && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gray-50 dark:bg-daily-light rounded-xl p-5 border border-gray-200 dark:border-gray-800"
        >
          <h4 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-3">{t('insights.weeklyBreakdown')}</h4>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs text-gray-400 dark:text-gray-500 border-b border-gray-200 dark:border-gray-700">
                  <th className="pb-2 pr-4 font-medium">{t('insights.weeklyWeek')}</th>
                  <th className="pb-2 pr-4 font-medium text-right">{t('insights.weeklySessions')}</th>
                  <th className="pb-2 pr-4 font-medium text-right">{t('insights.weeklyFriction')}</th>
                  <th className="pb-2 font-medium text-right">{t('insights.weeklySuccess')}</th>
                </tr>
              </thead>
              <tbody>
                {trends.weekly_stats.map((week, i) => (
                  <tr
                    key={i}
                    className="border-b border-gray-100 dark:border-gray-800 last:border-0"
                  >
                    <td className="py-2 pr-4 text-gray-600 dark:text-gray-300">{week.week_label}</td>
                    <td className="py-2 pr-4 text-right font-medium text-gray-900 dark:text-gray-100">
                      {week.session_count}
                    </td>
                    <td className="py-2 pr-4 text-right">
                      <span className={week.friction_rate > 50 ? 'text-red-500' : week.friction_rate > 25 ? 'text-yellow-500' : 'text-green-500'}>
                        {week.friction_rate.toFixed(0)}%
                      </span>
                    </td>
                    <td className="py-2 text-right">
                      <span className={week.success_rate >= 75 ? 'text-green-500' : week.success_rate >= 50 ? 'text-yellow-500' : 'text-red-500'}>
                        {week.success_rate.toFixed(0)}%
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </motion.div>
      )}
    </div>
  )
}
