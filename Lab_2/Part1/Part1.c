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
static uint32_t ledVal = 1;
/*************************************************
* function declarations
*************************************************/
int main(void);

/*************************************************
* timer 2 interrupt handler
*************************************************/
void TIM2_IRQHandler(void)
{
    
    // clear interrupt status
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }

    GPIOD->ODR = (ledVal << 12);

    if (ledVal == 0x08) {
        ledVal = 1;
    }
    else {
        ledVal = (ledVal << 1);
    }
}


void EXTI0_IRQHandler(void)
{

    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 0))
    {
        EXTI->PR = (1 << 0);
        while(!(EXTI->PR & (1 << 0)))
        {  
        }
        ledVal = 1;
        GPIOD->ODR = (ledVal << 12);
        EXTI->PR = (1 << 0);
        
    }

    return;
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_100();

    // setup LEDs
    RCC->AHB1ENR |= (1 << 3);
    GPIOD->MODER &= 0x00FFFFFF;
    GPIOD->MODER |= 0x55000000;
    GPIOD->ODR = 0;

       /* set up button */
    // enable GPIOA clock (AHB1ENR: bit 0)
    RCC->AHB1ENR |= (1 << 0);
    GPIOA->MODER &= 0xFFFFFFFC;   // Reset bits 0-1 to clear old values
    GPIOA->MODER |= 0x00000000;   // Make button an input

    // enable TIM2 clock (bit0)
    RCC->APB1ENR |= (1 << 0);

    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    // Timer clock runs at ABP1 * 2
    //   since ABP1 is set to /4 of fCLK
    //   thus 168M/4 * 2 = 84Mhz //nope 100M/4 * 2 = 50mHz
    // set prescaler to 83999 //maybe 49999
    //   it will increment counter every prescalar cycles
    // fCK_PSC / (PSC[15:0] + 1)
    // 84 Mhz / 8399 + 1 = 10 khz timer clock speed
    TIM2->PSC = 4999;

    // Set the auto-reload value to 10000
    //   which should give 1 second timer interrupts
    TIM2->ARR = 5000; //this theoreticall gives .12 second interupts,
                      //but produced 0.5 second "light up times"
                      
    // Update Interrupt Enable
    TIM2->DIER |= (1 << 0);

        /* tie push button at PA0 to EXTI0 */
    // EXTI0 can be configured for each GPIO module.
    //   EXTICR1: 0b XXXX XXXX XXXX 0000
    //               pin3 pin2 pin1 pin0
    //
    //   Writing a 0b0000 to pin0 location ties PA0 to EXT0
    SYSCFG->EXTICR[0] |= 0x00000000; // Write 0000 to map PA0 to EXTI0

    // Choose either rising edge trigger (RTSR) or falling edge trigger (FTSR)
    EXTI->RTSR |= 0x00001;   // Enable rising edge trigger on EXTI0
    
    EXTI->FTSR |= 0x00001;

    // Mask the used external interrupt numbers.
    EXTI->IMR |= 0x00001;    // Mask EXTI0

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI0_IRQn, 1); // Priority level 1

    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI0_IRQn);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    // Enable Timer 2 module (CEN, bit0)
    TIM2->CR1 |= (1 << 0);

    while(1)
    {
        // Do nothing.
    }

    return 0;
}
