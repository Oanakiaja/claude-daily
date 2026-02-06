import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  AreaChart,
  Area,
  CartesianGrid,
} from 'recharts'
import { useApi } from '../hooks/useApi'
import type { InsightsData } from '../hooks/useApi'
import { InsightsProblemView } from '../components/InsightsProblemView'
import { InsightsTrends } from '../components/InsightsTrends'

const COLORS = ['#f97316', '#fb923c', '#fdba74', '#fed7aa', '#ffedd5', '#a3e635', '#4ade80', '#2dd4bf', '#38bdf8', '#818cf8']
const FRICTION_COLORS = ['#ef4444', '#f97316', '#eab308', '#a3a3a3']

function StatCard({ label, value, sub }: { label: string; value: string | number; sub?: string }) {
  return (
    <div className="bg-gray-50 dark:bg-daily-light rounded-xl p-5 border border-gray-200 dark:border-gray-800">
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">{label}</p>
      <p className="text-3xl font-bold text-orange-500 dark:text-orange-400">{value}</p>
      {sub && <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">{sub}</p>}
    </div>
  )
}

function ChartCard({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-gray-50 dark:bg-daily-light rounded-xl p-5 border border-gray-200 dark:border-gray-800"
    >
      <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-4">{title}</h3>
      {children}
    </motion.div>
  )
}

// Custom tooltip styling
function CustomTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 shadow-lg">
      <p className="text-xs text-gray-500 dark:text-gray-400">{label}</p>
      {payload.map((p: any, i: number) => (
        <p key={i} className="text-sm font-medium" style={{ color: p.color }}>
          {p.value} {p.name || 'sessions'}
        </p>
      ))}
    </div>
  )
}

export function Insights() {
  const [data, setData] = useState<InsightsData | null>(null)
  const [loading, setLoading] = useState(true)
  const [days, setDays] = useState(30)
  const { fetchInsights } = useApi()

  useEffect(() => {
    const load = async () => {
      setLoading(true)
      try {
        const insights = await fetchInsights(days)
        setData(insights)
      } catch (err) {
        console.error('Failed to load insights:', err)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [fetchInsights, days])

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto px-6 py-8">
        <h1 className="text-3xl font-bold mb-8">
          <span className="text-orange-500 dark:text-orange-400">Insights</span>
        </h1>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-28 bg-gray-200 dark:bg-daily-light rounded-xl animate-pulse" />
          ))}
        </div>
        <div className="h-64 bg-gray-200 dark:bg-daily-light rounded-xl animate-pulse" />
      </div>
    )
  }

  if (!data) {
    return (
      <div className="max-w-6xl mx-auto px-6 py-8">
        <h1 className="text-3xl font-bold mb-4">
          <span className="text-orange-500 dark:text-orange-400">Insights</span>
        </h1>
        <p className="text-gray-500">Failed to load insights data.</p>
      </div>
    )
  }

  const avgSessions = data.total_days > 0
    ? (data.total_sessions / data.total_days).toFixed(1)
    : '0'

  const digestRate = data.total_days > 0
    ? Math.round((data.daily_stats.filter(d => d.has_digest).length / data.total_days) * 100)
    : 0

  return (
    <div className="max-w-6xl mx-auto px-6 py-8">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold">
            <span className="text-orange-500 dark:text-orange-400">Insights</span>
          </h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            Work pattern analysis across your sessions
          </p>
        </div>
        <div className="flex gap-2">
          {[7, 14, 30, 90].map(d => (
            <button
              key={d}
              onClick={() => setDays(d)}
              className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
                days === d
                  ? 'bg-orange-500 text-white'
                  : 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'
              }`}
            >
              {d}d
            </button>
          ))}
        </div>
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <StatCard label="Total Days" value={data.total_days} />
        <StatCard label="Total Sessions" value={data.total_sessions} />
        <StatCard label="Avg Sessions/Day" value={avgSessions} />
        <StatCard label="Digest Rate" value={`${digestRate}%`} sub="days with summary" />
      </div>

      {/* Trend Analysis */}
      <InsightsTrends trends={data.trends} />

      {/* Activity Timeline */}
      <ChartCard title="Activity Timeline">
        <ResponsiveContainer width="100%" height={200}>
          <AreaChart data={data.daily_stats}>
            <defs>
              <linearGradient id="colorSessions" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#f97316" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#f97316" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
            <XAxis
              dataKey="date"
              tick={{ fontSize: 11, fill: '#9ca3af' }}
              tickFormatter={(v: string) => v.slice(5)}
            />
            <YAxis tick={{ fontSize: 11, fill: '#9ca3af' }} allowDecimals={false} />
            <Tooltip content={<CustomTooltip />} />
            <Area
              type="monotone"
              dataKey="session_count"
              stroke="#f97316"
              strokeWidth={2}
              fill="url(#colorSessions)"
            />
          </AreaChart>
        </ResponsiveContainer>
      </ChartCard>

      {/* Distribution Charts */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
        {/* Goal Distribution */}
        {data.goal_distribution.length > 0 && (
          <ChartCard title="Goal Distribution">
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={data.goal_distribution}
                  dataKey="count"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  label={({ name, percent }: any) => `${name} ${(percent * 100).toFixed(0)}%`}
                  labelLine={{ stroke: '#9ca3af' }}
                >
                  {data.goal_distribution.map((_, i) => (
                    <Cell key={i} fill={COLORS[i % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip content={<CustomTooltip />} />
              </PieChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {/* Satisfaction Distribution */}
        {data.satisfaction_distribution.length > 0 && (
          <ChartCard title="Satisfaction">
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={data.satisfaction_distribution} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                <XAxis type="number" tick={{ fontSize: 11, fill: '#9ca3af' }} allowDecimals={false} />
                <YAxis
                  type="category"
                  dataKey="name"
                  tick={{ fontSize: 12, fill: '#9ca3af' }}
                  width={100}
                />
                <Tooltip content={<CustomTooltip />} />
                <Bar dataKey="count" radius={[0, 4, 4, 0]}>
                  {data.satisfaction_distribution.map((entry, i) => {
                    const color = entry.name === 'happy' ? '#4ade80'
                      : entry.name === 'satisfied' ? '#a3e635'
                      : entry.name === 'neutral' ? '#eab308'
                      : entry.name === 'frustrated' ? '#ef4444'
                      : COLORS[i % COLORS.length]
                    return <Cell key={i} fill={color} />
                  })}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {/* Friction Points */}
        {data.friction_distribution.length > 0 && (
          <ChartCard title="Friction Points">
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={data.friction_distribution} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                <XAxis type="number" tick={{ fontSize: 11, fill: '#9ca3af' }} allowDecimals={false} />
                <YAxis
                  type="category"
                  dataKey="name"
                  tick={{ fontSize: 12, fill: '#9ca3af' }}
                  width={120}
                />
                <Tooltip content={<CustomTooltip />} />
                <Bar dataKey="count" radius={[0, 4, 4, 0]}>
                  {data.friction_distribution.map((_, i) => (
                    <Cell key={i} fill={FRICTION_COLORS[i % FRICTION_COLORS.length]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {/* Languages */}
        {data.language_distribution.length > 0 && (
          <ChartCard title="Languages">
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={data.language_distribution.slice(0, 8)} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                <XAxis type="number" tick={{ fontSize: 11, fill: '#9ca3af' }} allowDecimals={false} />
                <YAxis
                  type="category"
                  dataKey="name"
                  tick={{ fontSize: 12, fill: '#9ca3af' }}
                  width={100}
                />
                <Tooltip content={<CustomTooltip />} />
                <Bar dataKey="count" fill="#38bdf8" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}
      </div>

      {/* Session Types */}
      {data.session_type_distribution.length > 0 && (
        <div className="mt-4">
          <ChartCard title="Session Types">
            <div className="flex flex-wrap gap-3">
              {data.session_type_distribution.map((item, i) => (
                <div
                  key={item.name}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700"
                >
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: COLORS[i % COLORS.length] }}
                  />
                  <span className="text-sm text-gray-600 dark:text-gray-300">{item.name}</span>
                  <span className="text-sm font-bold text-orange-500 dark:text-orange-400">{item.count}</span>
                </div>
              ))}
            </div>
          </ChartCard>
        </div>
      )}

      {/* Empty state for facets */}
      {data.goal_distribution.length === 0 && data.friction_distribution.length === 0 && (
        <div className="mt-8 text-center py-12 bg-gray-50 dark:bg-daily-light rounded-xl border border-gray-200 dark:border-gray-800">
          <p className="text-gray-500 dark:text-gray-400 mb-2">No facet data available yet</p>
          <p className="text-sm text-gray-400 dark:text-gray-500">
            Run <code className="px-2 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-orange-500">/insights</code> in Claude Code to generate detailed analysis
          </p>
        </div>
      )}

      {/* Problem-focused session details */}
      {data.session_details && data.session_details.length > 0 && (
        <InsightsProblemView sessionDetails={data.session_details} />
      )}
    </div>
  )
}
