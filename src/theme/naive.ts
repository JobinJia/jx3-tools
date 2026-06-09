import type { GlobalThemeOverrides } from 'naive-ui'

/** 与 src/assets/theme.css 的 CSS 变量保持同一份色值 */
export const lightOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#9c2f23',
    primaryColorHover: '#b03a2e',
    primaryColorPressed: '#7e251c',
    primaryColorSuppl: '#b03a2e',
    successColor: '#5a7247',
    warningColor: '#b07936',
    errorColor: '#8f2727',
    infoColor: '#44617b',
    textColorBase: '#2c2a26',
    textColor1: '#2c2a26',
    textColor2: '#6f675a',
    textColor3: '#a09a8c',
    bodyColor: '#f4eedd',
    cardColor: '#fffdf6',
    popoverColor: '#fffdf6',
    modalColor: '#fffdf6',
    inputColor: '#f7f2e4',
    borderColor: '#ddd3bb',
    dividerColor: '#efe8d4',
    borderRadius: '6px',
  },
}

export const darkOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#c24a3a',
    primaryColorHover: '#d05c4b',
    primaryColorPressed: '#a63c2e',
    primaryColorSuppl: '#d05c4b',
    successColor: '#7e9a68',
    warningColor: '#c99850',
    errorColor: '#c0564a',
    infoColor: '#7395b3',
    textColorBase: '#e8e0cd',
    textColor1: '#e8e0cd',
    textColor2: '#b3a994',
    textColor3: '#7d7665',
    bodyColor: '#211f1b',
    cardColor: '#2b2823',
    popoverColor: '#2b2823',
    modalColor: '#2b2823',
    inputColor: '#26231f',
    borderColor: '#3d382f',
    dividerColor: '#353027',
    borderRadius: '6px',
  },
}
