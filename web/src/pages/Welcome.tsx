export function Welcome() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <div className="text-center space-y-4">
          <h1 className="text-4xl font-bold text-balance mb-2">
            Welcome to <span className="text-orange-500 dark:text-orange-400">Daily</span>
          </h1>
          <p className="text-gray-500 dark:text-gray-400 text-lg max-w-md mx-auto">
            Your context archive system for Claude Code sessions
          </p>
          <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-800">
            <p className="text-gray-500 text-sm">
              Select a date from the left sidebar to view your archives
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
