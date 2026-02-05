import { NavLink, Outlet } from 'react-router-dom'
import { cn } from '../lib/utils'
import { ThemeToggle } from './ThemeToggle'

export function Layout() {
  return (
    <div className="min-h-dvh bg-white dark:bg-daily-dark text-gray-900 dark:text-gray-100 transition-colors">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 bg-white/90 dark:bg-daily-dark/90 backdrop-blur-md border-b border-gray-200 dark:border-orange-500/20 transition-colors">
        <div className="px-6 py-3 flex items-center gap-6">
          <NavLink to="/" className="text-xl font-bold text-orange-500 dark:text-orange-400 hover:text-orange-600 dark:hover:text-orange-300 transition-colors">
            Daily
          </NavLink>
          <nav className="flex gap-4 flex-1">
            <NavLink
              to="/"
              end
              className={({ isActive }) =>
                cn(
                  'text-sm transition-colors',
                  isActive
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
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
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                )
              }
            >
              Jobs
            </NavLink>
            <NavLink
              to="/settings"
              className={({ isActive }) =>
                cn(
                  'text-sm transition-colors',
                  isActive
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                )
              }
            >
              Settings
            </NavLink>
          </nav>
          <ThemeToggle />
        </div>
      </header>

      {/* Main Content */}
      <main className="pt-16 min-h-dvh">
        <Outlet />
      </main>
    </div>
  )
}
