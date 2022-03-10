/*
 *   blinks LEDs one at a time (order: green, orange, red, blue) using timer interrupt
 *   timer2 is used as the source, and it is setup
 *   to run at 10 kHz. LED blinking rate is set to
 *   0.5 second.
 * 
 *   Uses PC7 as a touch sensor, one touch pauses the sequence, the
 *   second touch resumes the sequence from the green led
 *
 *   See lab report for sources
 */

#include "stm32f4xx.h"
#include "system_stm32f4xx.h"
static uint32_t ledVal = 1;
static uint32_t blinking = 1;
/*************************************************
* function declarations
*************************************************/
int main(void);

/*************************************************
* exti 7 interrupt handler
*************************************************/

void EXTI9_5_IRQHandler(void)
{
    // Check if the interrupt came from exti7
    if (EXTI->PR & (1 << 7))
    {
            if(blinking == 1) //if the sequence is going, pause it
            {
                blinking = 0;
            }
            else
            {
                blinking = 1; //if the sequence is paused, start it up from green again
                ledVal = 1;
            }
        EXTI->PR = (1 << 7);
    }
    return;
}

/*************************************************
* timer 3 interrupt handler
*************************************************/
void TIM3_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    
    // clear interrupt status
    if (TIM3->DIER & 0x01) {
        if (TIM3->SR & 0x01) {
            TIM3->SR &= ~(1U << 0);
        }
    }

    GPIOC->MODER |= (1 << 14);  
    GPIOC->ODR |= (1 << 7);
    GPIOC->MODER &= ~(1U << 14);  
}

/*************************************************
* timer 2 interrupt handler
*************************************************/
void TIM2_IRQHandler(void) //light up each led sequentitally, called every 0.5 seconds
{
    
    // clear interrupt status
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }
    if(blinking == 1) //only light up if sequence isn't paused
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

    /* set up PC7 */
    // enable GPIOC clock (AHB1ENR: bit 2)
    RCC->AHB1ENR |= (1 << 2);
    GPIOC->MODER &= ~((1U << 14) | (1U << 15));   // Reset bits 14-15 to clear old values (line 7)
    GPIOC->MODER |= 0x00000000;   // Make PC7 an input
    //by default, PC7 is floating
    
    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    // enable TIM3 clock (bit1)
    RCC->APB1ENR |= (1 << 1);

    // enable TIM2 clock (bit0)
    RCC->APB1ENR |= (1 << 0);

    // Timer clock runs at ABP1 * 2
    //   since ABP1 is set to /4 of fCLK
    //   thus 100M/4 * 2 = 50Mhz 
    // set prescaler to 49999 
    //   it will increment counter every prescalar cycles
    // fCK_PSC / (PSC[15:0] + 1)
    // 50 Mhz / 4999 + 1 = 10 khz timer clock speed
    TIM3->PSC = 4999; //set TIM3 prescalar
    TIM3->ARR = 150 * 20; //set auto refil value to 0.3 seconds
    TIM3->DIER |= (1 << 0);  //enable TIM3 interrupt

    TIM2->PSC = 4999; //set TIM2 prescalar
    TIM2->ARR = 5000; //set auto refil value to 0.5 seconds
    TIM2->DIER |= (1 << 0); //enable TIM2 interrupt

    /* tie PC7 to EXTI7 */
    // EXTI7 can be configured for each GPIO module.
    //   EXTICR2: 0b XXXX XXXX XXXX 0000
    //               pin7 pin6 pin5 pin4
    //
    //   Writing a 0b0010 to pin7 (bits 12-15) location ties PC7 to EXTI7
    SYSCFG->EXTICR[1] &= ~((1U << 12) | (1U << 13) | (1U << 14) | (1U << 15)); // Write 0010 to map PC7 to EXTI7
    SYSCFG->EXTICR[1] |= (1U << 13);
    EXTI->FTSR |= (1 << 7);   // Enable falling edge trigger on EXTI0

    // Mask the used external interrupt numbers.
    EXTI->IMR |= (1 << 7);    // Mask EXTI7

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI9_5_IRQn, 1); // Priority level 1

    // enable EXT7 IRQ from NVIC
    NVIC_EnableIRQ(EXTI9_5_IRQn);

    NVIC_SetPriority(TIM3_IRQn, 3); // Priority level 2
    // enable TIM3 IRQ from NVIC
    NVIC_EnableIRQ(TIM3_IRQn);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    // Enable Timer 2 and 3 module (CEN, bit0)
    TIM3->CR1 |= (1 << 0);
    TIM2->CR1 |= (1 << 0);

    while(1)
    {
    }

    return 0;
}
