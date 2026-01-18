declare module 'react-scrollama' {
  import { ReactNode, FC } from 'react';

  export interface StepProps {
    data?: number | string | object;
    children: ReactNode;
  }

  export interface ScrollamaProps {
    offset?: number;
    onStepEnter?: (response: { data: number | string | object; entry: IntersectionObserverEntry }) => void;
    onStepExit?: (response: { data: number | string | object; entry: IntersectionObserverEntry; direction: string }) => void;
    onStepProgress?: (response: { data: number | string | object; progress: number; entry: IntersectionObserverEntry }) => void;
    children: ReactNode;
  }

  export const Step: FC<StepProps>;
  export const Scrollama: FC<ScrollamaProps>;
}
