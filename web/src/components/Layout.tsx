import { NavLink, Outlet } from 'react-router-dom'
import { cn } from '../lib/utils'

export function Layout() {
  return (
    <div className="min-h-dvh bg-daily-dark text-gray-100">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 bg-daily-dark/90 backdrop-blur-md border-b border-orange-500/20">
        <div className="max-w-7xl mx-auto px-6 py-3 flex items-center justify-between">
          <div className="flex items-center gap-6">
            <NavLink to="/" className="text-xl font-bold text-orange-400 hover:text-orange-300 transition-colors">
              Daily
            </NavLink>
            <nav className="flex gap-4">
              <NavLink
                to="/"
                end
                className={({ isActive }) =>
                  cn(
                    'text-sm transition-colors',
                    isActive
                      ? 'text-orange-400'
                      : 'text-gray-400 hover:text-gray-200'
                  )
                }
              >
                Archives
              </NavLink>
              <NavLink
                to="/jobs"
                className={({ isActive }) =>
                  cn(
                    'text-sm transition-colors',
                    isActive
                      ? 'text-orange-400'
                      : 'text-gray-400 hover:text-gray-200'
                  )
                }
              >
                Jobs
              </NavLink>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="pt-16 min-h-dvh">
        <Outlet />
      </main>
    </div>
  )
}
