/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        talker: {
          light: {
            bg: '#FFFFFF',
            surface: '#F7F7F7',
            secondary: '#F2F2F2',
            border: '#E6E6E6',
            text: '#111111',
            muted: '#6D6D6D',
          },
          dark: {
            bg: '#202020',
            surface: '#282828',
            elevated: '#303030',
            border: '#3A3A3A',
            text: '#F2F2F2',
            muted: '#B5B5B5',
          },
        },
      },
      fontFamily: {
        sans: [
          'Segoe UI Variable Text',
          'Segoe UI',
          '-apple-system',
          'BlinkMacSystemFont',
          'Roboto',
          'Helvetica Neue',
          'sans-serif',
        ],
        mono: [
          'Cascadia Code',
          'Consolas',
          'monospace',
        ],
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'slide-down': {
          '0%': { opacity: '0', transform: 'translateY(-4px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-up': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'blink': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.15s ease-out',
        'slide-down': 'slide-down 0.18s ease-out',
        'slide-up': 'slide-up 0.25s ease-out',
        'blink': 'blink 1.1s ease-in-out infinite',
      },
    },
  },
  plugins: [],
};
