/*
 * timer.c
 *
 * author: Furkan Cayci
 * description:
 *   blinks LEDs one at a time using timer interrupt
 *   timer2 is used as the source, and it is setup
 *   to run at 10 kHz. LED blinking rate is set to
 *   1 second.
 *
 * timer and timer interrupt setup steps:
 *   1. Enable TIMx clock from RCC
 *   2. Set prescaler for the timer from PSC
 *   3. Set auto-reload value from ARR
 *   4. (optional) Enable update interrupt from DIER bit 0
 *   5. (optional) Enable TIMx interrupt from NVIC
 *   6. Enable TIMx module from CR1 bit 0
 */

#include "stm32f4xx.h"
#include "system_stm32f4xx.h"
#define THRESHOLD (2)
static uint32_t ledVal = 1;
static uint32_t  timePassed = 0;
static uint32_t blinking = 1;
static uint32_t state = 0;
/*************************************************
* function declarations
*************************************************/
int main(void);

/*************************************************
* timer 2 interrupt handler
*************************************************/

void EXTI9_5_IRQHandler(void) //TODO
{
    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 7))
    {
        if(timePassed >= THRESHOLD)
        {
            state++;
        }
        else if (state > 10)
        {
            state = 0;
            if(blinking == 1)
            {
                blinking = 0;
            }
            else
            {
                blinking = 1;
                ledVal = 1;
                GPIOD->ODR = (ledVal << 12);
            }
        }
        EXTI->PR = (1 << 7);
            //GPIOC->ODR |= (1 << 7);
    }
    return;
}

void TIM3_IRQHandler(void)
{
    
    // clear interrupt status
    if (TIM3->DIER & 0x01) {
        if (TIM3->SR & 0x01) {
            TIM3->SR &= ~(1U << 0);
        }
    }
    if(timePassed > 800)
    {
        timePassed = 0;
        GPIOC->MODER |= (1 << 14); 
        GPIOC->ODR |= (1 << 7);
        GPIOC->MODER &= ~(1U << 14); 
    }
    
    timePassed++;  
}

void TIM2_IRQHandler(void)
{
    
    // clear interrupt status
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }
    if(blinking == 1)
    {
        GPIOD->ODR = (ledVal << 12);

        if (ledVal == 0x08) {
            ledVal = 1;
        }
        else {
            ledVal = (ledVal << 1);
        }
    }
    
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 100 Mhz */
    set_sysclk_to_100();

    /* setup LEDs */
    // enable GPIOD clock (AHB1ENR: bit 3)
    RCC->AHB1ENR |= (1 << 3);
    GPIOD->MODER &= 0x00FFFFFF;   // Reset bits 31-24 to clear old values
    GPIOD->MODER |= 0x55000000;   // Write 01 for all 4 leds to make them output

    /* set up button */
    // enable GPIOC clock (AHB1ENR: bit 2)
    RCC->AHB1ENR |= (1 << 2);
    GPIOC->MODER &= ~((1U << 14) | (1U << 15));   // Reset bits 14-15 to clear old values (line 7)
    GPIOC->MODER |= 0x00000000;   // Make button an input
    GPIOC->PUPDR |= (1 << 15);
    GPIOC->PUPDR &= ~(1U << 14);
    
    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    // enable TIM3 clock (bit1)
    RCC->APB1ENR |= (1 << 1);

    // enable TIM2 clock (bit0)
     RCC->APB1ENR |= (1 << 0);

    TIM3->PSC = 4999;
    TIM3->ARR = 50;
    TIM3->DIER |= (1 << 1); 

    TIM2->PSC = 4999;
    TIM2->ARR = 5000;
    TIM2->DIER |= (1 << 0);

    /* tie push button at PA0 to EXTI7 */
    // EXTI0 can be configured for each GPIO module.
    //   EXTICR1: 0b XXXX XXXX XXXX 0000
    //               pin3 pin2 pin1 pin0
    //
    //   Writing a 0b0000 to pin0 location ties PA0 to EXT0
    SYSCFG->EXTICR[1] &= ~((1U << 12) | (1U << 13) | (1U << 14) | (1U << 15)); // Write 0000 to map PA0 to EXTI0
    SYSCFG->EXTICR[1] |= (1U << 13);
    // Choose either rising edge trigger (RTSR) or falling edge trigger (FTSR)
    EXTI->FTSR |= (1 << 7);   // Enable rising edge trigger on EXTI0

    // Mask the used external interrupt numbers.
    EXTI->IMR |= (1 << 7);    // Mask EXTI0

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI9_5_IRQn, 1); // Priority level 1

    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI9_5_IRQn);

    NVIC_SetPriority(TIM3_IRQn, 3); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM3_IRQn);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    // Enable Timer 2 module (CEN, bit0)
    TIM3->CR1 |= (1 << 0);
    TIM2->CR1 |= (1 << 0);

    while(1)
    {
    }

    return 0;
}
