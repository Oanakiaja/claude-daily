import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
  PieChart,
  Pie,
  Cell,
} from 'recharts'
import type { DailyUsageData, ModelUsageCount } from '../hooks/useApi'

const MODEL_COLORS = ['#f97316', '#38bdf8', '#a78bfa', '#4ade80', '#fb923c', '#818cf8', '#2dd4bf', '#f472b6']

export function formatTokenCount(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return n.toString()
}

export function formatCost(usd: number): string {
  if (usd >= 1) return `$${usd.toFixed(2)}`
  if (usd >= 0.01) return `$${usd.toFixed(3)}`
  return `$${usd.toFixed(4)}`
}

function UsageTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 shadow-lg">
      <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">{label}</p>
      {payload.map((p: any, i: number) => (
        <p key={i} className="text-sm font-medium" style={{ color: p.color }}>
          {p.name}: {typeof p.value === 'number' && p.name?.includes('cost') ? formatCost(p.value) : formatTokenCount(p.value)}
        </p>
      ))}
    </div>
  )
}

function CostTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 shadow-lg">
      <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">{label}</p>
      {payload.map((p: any, i: number) => (
        <p key={i} className="text-sm font-medium" style={{ color: p.color }}>
          {formatCost(p.value)}
        </p>
      ))}
    </div>
  )
}

export function UsageTimeline({ data }: { data: DailyUsageData[] }) {
  return (
    <ResponsiveContainer width="100%" height={220}>
      <AreaChart data={data}>
        <defs>
          <linearGradient id="colorInput" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="#38bdf8" stopOpacity={0.3} />
            <stop offset="95%" stopColor="#38bdf8" stopOpacity={0} />
          </linearGradient>
          <linearGradient id="colorOutput" x1="0" y1="0" x2="0" y2="1">
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
        <YAxis
          tick={{ fontSize: 11, fill: '#9ca3af' }}
          tickFormatter={formatTokenCount}
        />
        <Tooltip content={<UsageTooltip />} />
        <Area
          type="monotone"
          dataKey="input_tokens"
          name="input"
          stackId="1"
          stroke="#38bdf8"
          strokeWidth={1.5}
          fill="url(#colorInput)"
        />
        <Area
          type="monotone"
          dataKey="output_tokens"
          name="output"
          stackId="1"
          stroke="#f97316"
          strokeWidth={1.5}
          fill="url(#colorOutput)"
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}

export function CostTimeline({ data }: { data: DailyUsageData[] }) {
  return (
    <ResponsiveContainer width="100%" height={220}>
      <AreaChart data={data}>
        <defs>
          <linearGradient id="colorCost" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="#a78bfa" stopOpacity={0.3} />
            <stop offset="95%" stopColor="#a78bfa" stopOpacity={0} />
          </linearGradient>
        </defs>
        <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
        <XAxis
          dataKey="date"
          tick={{ fontSize: 11, fill: '#9ca3af' }}
          tickFormatter={(v: string) => v.slice(5)}
        />
        <YAxis
          tick={{ fontSize: 11, fill: '#9ca3af' }}
          tickFormatter={(v: number) => formatCost(v)}
        />
        <Tooltip content={<CostTooltip />} />
        <Area
          type="monotone"
          dataKey="total_cost_usd"
          name="cost"
          stroke="#a78bfa"
          strokeWidth={2}
          fill="url(#colorCost)"
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}

export function ModelPieChart({ data }: { data: ModelUsageCount[] }) {
  if (!data || data.length === 0) return null

  // Simplify model names for display
  const chartData = data.map(d => ({
    ...d,
    displayName: simplifyModelName(d.model),
  }))

  return (
    <ResponsiveContainer width="100%" height={250}>
      <PieChart>
        <Pie
          data={chartData}
          dataKey="count"
          nameKey="displayName"
          cx="50%"
          cy="50%"
          outerRadius={80}
          label={({ displayName, percent }: any) =>
            `${displayName} ${(percent * 100).toFixed(0)}%`
          }
          labelLine={{ stroke: '#9ca3af' }}
        >
          {chartData.map((_, i) => (
            <Cell key={i} fill={MODEL_COLORS[i % MODEL_COLORS.length]} />
          ))}
        </Pie>
        <Tooltip
          formatter={(value: number, name: string) => [value, name]}
        />
      </PieChart>
    </ResponsiveContainer>
  )
}

function simplifyModelName(model: string): string {
  if (model.includes('opus')) return 'Opus'
  if (model.includes('sonnet')) return 'Sonnet'
  if (model.includes('haiku')) return 'Haiku'
  return model
}
