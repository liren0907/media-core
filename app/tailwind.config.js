/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        glass: {
          'primary': 'rgba(255, 255, 255, 0.6)',
          'secondary': 'rgba(255, 255, 255, 0.4)',
          'dark': 'rgba(30, 41, 59, 0.8)', // Slate 800
          'accent': 'rgba(249, 115, 22, 0.9)', // Orange 500
          'accent-light': 'rgba(251, 146, 60, 0.7)', // Orange 400
        }
      },
      backdropBlur: {
        'glass': '12px',
        'glass-lg': '20px',
      },
      boxShadow: {
        'glass': '0 8px 32px 0 rgba(249, 115, 22, 0.08)', // Warm shadow
        'glass-sm': '0 4px 16px 0 rgba(249, 115, 22, 0.05)',
        'glass-lg': '0 12px 48px 0 rgba(249, 115, 22, 0.12)',
        'glass-inset': 'inset 0 2px 4px 0 rgba(255, 255, 255, 0.6)',
        'glass-glow': '0 0 20px rgba(251, 146, 60, 0.3)', // Warm glow
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'glass-gradient': 'linear-gradient(135deg, rgba(255,255,255,0.8) 0%, rgba(255,255,255,0.4) 100%)',
      },
      animation: {
        'float': 'float 6s ease-in-out infinite',
        'glow': 'glow 3s ease-in-out infinite alternate',
        'shimmer': 'shimmer 2s linear infinite',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0px)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        glow: {
          'from': { boxShadow: '0 0 10px rgba(251, 146, 60, 0.2)' },
          'to': { boxShadow: '0 0 20px rgba(251, 146, 60, 0.4)' },
        },
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
      },
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
    require('daisyui'),
  ],
}
