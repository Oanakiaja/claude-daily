export interface CommandOption {
  flag: string;
  desc: string;
}

export interface CommandExample {
  cmd: string;
  desc: string;
}

export interface CommandSubcommand {
  cmd: string;
  desc: string;
}

export interface Command {
  name: string;
  usage: string;
  description: string;
  options?: CommandOption[];
  subcommands?: CommandSubcommand[];
  examples?: CommandExample[];
}

export interface SlashCommand {
  name: string;
  description: string;
  usage: string;
}

export interface QuickStartItem {
  step: number;
  title: string;
  cmd: string;
}

export interface GlobalOption {
  flag: string;
  desc: string;
}

export const commands: Command[] = [
  {
    name: 'init',
    usage: 'daily init [-i|--interactive] [-s|--storage-path <PATH>]',
    description: 'Initialize Daily configuration. Sets up the storage directory and default settings.',
    options: [
      { flag: '-i, --interactive', desc: 'Interactive mode: fuzzy search and select directory' },
      { flag: '-s, --storage-path <PATH>', desc: 'Storage path (default: ~/.claude/daily)' },
    ],
    examples: [
      { cmd: 'daily init', desc: 'Use default settings' },
      { cmd: 'daily init -i', desc: 'Interactive setup with directory selection' },
      { cmd: 'daily init -s ~/my-logs', desc: 'Custom storage path' },
    ],
  },
  {
    name: 'install',
    usage: 'daily install [-s|--scope <SCOPE>]',
    description: 'Install Daily hooks and slash commands to Claude Code. This enables automatic session recording.',
    options: [
      { flag: '-s, --scope <SCOPE>', desc: 'Installation scope: "user" or "project" (default: user)' },
    ],
    examples: [
      { cmd: 'daily install', desc: 'Install for current user' },
      { cmd: 'daily install --scope project', desc: 'Install for current project only' },
    ],
  },
  {
    name: 'view',
    usage: 'daily view [-d|--date <DATE>] [--summary-only] [--list]',
    description: 'View archives. Opens interactive date selection if no date specified.',
    options: [
      { flag: '-d, --date <DATE>', desc: 'Date to view (format: yyyy-mm-dd)' },
      { flag: '--summary-only', desc: 'Show daily summary only' },
      { flag: '--list', desc: 'List all sessions for the day' },
    ],
    examples: [
      { cmd: 'daily view', desc: 'Interactive date selection' },
      { cmd: 'daily view -d 2024-01-15', desc: 'View specific date' },
      { cmd: 'daily view --list', desc: 'List all sessions' },
    ],
  },
  {
    name: 'today',
    usage: 'daily today [--summary-only] [--list]',
    description: "Quick alias to view today's archive.",
    options: [
      { flag: '--summary-only', desc: 'Show daily summary only' },
      { flag: '--list', desc: 'List all sessions' },
    ],
    examples: [
      { cmd: 'daily today', desc: "View today's archive" },
      { cmd: 'daily today --list', desc: "List today's sessions" },
    ],
  },
  {
    name: 'yest',
    usage: 'daily yest [--summary-only] [--list]',
    description: "Quick alias to view yesterday's archive.",
    options: [
      { flag: '--summary-only', desc: 'Show daily summary only' },
      { flag: '--list', desc: 'List all sessions' },
    ],
    examples: [
      { cmd: 'daily yest', desc: "View yesterday's archive" },
    ],
  },
  {
    name: 'digest',
    usage: 'daily digest [RELATIVE_DATE] [-d|--date <DATE>] [--background]',
    description: 'Consolidate session files into a single daily.md summary. Sessions are removed after digest.',
    options: [
      { flag: 'RELATIVE_DATE', desc: 'Relative date like "yest" or "yesterday"' },
      { flag: '-d, --date <DATE>', desc: 'Date to digest (format: yyyy-mm-dd, default: today)' },
      { flag: '--background', desc: 'Run in background (default: foreground)' },
    ],
    examples: [
      { cmd: 'daily digest', desc: "Digest today's sessions" },
      { cmd: 'daily digest yest', desc: "Digest yesterday's sessions" },
      { cmd: 'daily digest -d 2024-01-15', desc: 'Digest specific date' },
    ],
  },
  {
    name: 'extract-skill',
    usage: 'daily extract-skill [-d|--date <DATE>] [-s|--session <NAME>] [-o|--output <PATH>]',
    description: 'Extract reusable skill from a session archive. Creates a skill file that can be used in future sessions.',
    options: [
      { flag: '-d, --date <DATE>', desc: 'Date to search (default: today)' },
      { flag: '-s, --session <NAME>', desc: 'Session to extract from' },
      { flag: '-o, --output <PATH>', desc: 'Output directory for skill' },
    ],
    examples: [
      { cmd: 'daily extract-skill', desc: 'Interactive skill extraction' },
      { cmd: 'daily extract-skill -s my-session', desc: 'Extract from specific session' },
    ],
  },
  {
    name: 'extract-command',
    usage: 'daily extract-command [-d|--date <DATE>] [-s|--session <NAME>] [-o|--output <PATH>]',
    description: 'Extract command from a session archive. Creates a slash command for Claude Code.',
    options: [
      { flag: '-d, --date <DATE>', desc: 'Date to search (default: today)' },
      { flag: '-s, --session <NAME>', desc: 'Session to extract from' },
      { flag: '-o, --output <PATH>', desc: 'Output directory for command' },
    ],
    examples: [
      { cmd: 'daily extract-command', desc: 'Interactive command extraction' },
    ],
  },
  {
    name: 'review-skills',
    usage: 'daily review-skills [--install <REF>] [--delete <REF>]',
    description: 'Review and manage pending skills that were auto-extracted from sessions. Skills are extracted when sessions contain debugging discoveries or reusable solutions.',
    options: [
      { flag: '--install <REF>', desc: 'Install a pending skill (format: YYYY-MM-DD/skill-name)' },
      { flag: '--delete <REF>', desc: 'Delete a pending skill (format: YYYY-MM-DD/skill-name)' },
    ],
    examples: [
      { cmd: 'daily review-skills', desc: 'List all pending skills' },
      { cmd: 'daily review-skills --install 2024-01-18/fix-econnrefused', desc: 'Install a skill' },
      { cmd: 'daily review-skills --delete 2024-01-18/fix-econnrefused', desc: 'Delete a skill' },
    ],
  },
  {
    name: 'config',
    usage: 'daily config [--show] [--set-storage <PATH>]',
    description: 'Show or update Daily configuration.',
    options: [
      { flag: '--show', desc: 'Show current configuration' },
      { flag: '--set-storage <PATH>', desc: 'Set storage path' },
    ],
    examples: [
      { cmd: 'daily config --show', desc: 'Display current config' },
      { cmd: 'daily config --set-storage ~/logs', desc: 'Update storage path' },
    ],
  },
  {
    name: 'jobs',
    usage: 'daily jobs <ACTION>',
    description: 'Manage background summarization jobs.',
    subcommands: [
      { cmd: 'jobs list [-a|--all]', desc: 'List jobs (--all includes completed)' },
      { cmd: 'jobs log <JOB_ID> [-t|--tail N] [-f|--follow]', desc: 'Show job log' },
      { cmd: 'jobs kill <JOB_ID>', desc: 'Kill a running job' },
      { cmd: 'jobs cleanup [-d|--days N] [--dry-run]', desc: 'Cleanup old jobs (default: 7 days)' },
    ],
    examples: [
      { cmd: 'daily jobs list', desc: 'List running jobs' },
      { cmd: 'daily jobs list -a', desc: 'List all jobs' },
      { cmd: 'daily jobs log abc123 -f', desc: 'Follow job output' },
    ],
  },
  {
    name: 'show',
    usage: 'daily show [-p|--port <PORT>] [--host <HOST>] [--no-open]',
    description: 'Start web dashboard server for viewing archives. Opens browser by default.',
    options: [
      { flag: '-p, --port <PORT>', desc: 'Port to listen on (default: 3000)' },
      { flag: '--host <HOST>', desc: 'Host to bind to (default: 127.0.0.1)' },
      { flag: '--no-open', desc: 'Do not open browser automatically' },
    ],
    examples: [
      { cmd: 'daily show', desc: 'Start dashboard and open browser' },
      { cmd: 'daily show -p 8080', desc: 'Custom port' },
      { cmd: 'daily show --no-open', desc: 'Start without opening browser' },
    ],
  },
];

export const slashCommands: SlashCommand[] = [
  {
    name: '/daily-view',
    description: "View today's daily archive or a specific date",
    usage: 'Type /daily-view in Claude Code',
  },
  {
    name: '/daily-get-skill',
    description: 'Extract and generate a skill from daily archive insights',
    usage: 'Type /daily-get-skill in Claude Code',
  },
  {
    name: '/daily-get-command',
    description: 'Extract and generate a command from daily archive insights',
    usage: 'Type /daily-get-command in Claude Code',
  },
];

export const quickStart: QuickStartItem[] = [
  { step: 1, title: 'Install Daily', cmd: 'curl -fsSL https://raw.githubusercontent.com/oanakiaja/claude-daily/main/scripts/install.sh | bash' },
  { step: 2, title: 'Initialize', cmd: 'daily init -i' },
  { step: 3, title: 'Install hooks', cmd: 'daily install' },
  { step: 4, title: 'Start coding', cmd: 'claude' },
];

export const globalOptions: GlobalOption[] = [
  { flag: '-v, --verbose', desc: 'Enable verbose output' },
  { flag: '-c, --config <PATH>', desc: 'Custom config file path' },
  { flag: '-h, --help', desc: 'Show help information' },
  { flag: '-V, --version', desc: 'Show version' },
];
