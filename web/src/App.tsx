import { Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { DailyList } from './pages/DailyList'
import { DayDetail } from './pages/DayDetail'
import { SessionDetail } from './pages/SessionDetail'
import { JobsMonitor } from './pages/JobsMonitor'

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<DailyList />} />
        <Route path="day/:date" element={<DayDetail />} />
        <Route path="day/:date/session/:name" element={<SessionDetail />} />
        <Route path="jobs" element={<JobsMonitor />} />
      </Route>
    </Routes>
  )
}
